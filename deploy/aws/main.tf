data "aws_availability_zones" "available" {
  state = "available"
}

locals {
  az       = data.aws_availability_zones.available.names[0]
  registry = "${var.account_id}.dkr.ecr.${var.region}.amazonaws.com"
}

# ---- network: one public subnet, nothing else ----------------------------

resource "aws_vpc" "main" {
  cidr_block           = "10.42.0.0/24"
  enable_dns_support   = true
  enable_dns_hostnames = true
  tags                 = { Name = "dikumud" }
}

resource "aws_internet_gateway" "main" {
  vpc_id = aws_vpc.main.id
  tags   = { Name = "dikumud" }
}

resource "aws_subnet" "public" {
  vpc_id            = aws_vpc.main.id
  cidr_block        = "10.42.0.0/26"
  availability_zone = local.az
  tags              = { Name = "dikumud-public" }
}

resource "aws_route_table" "public" {
  vpc_id = aws_vpc.main.id
  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.main.id
  }
  tags = { Name = "dikumud-public" }
}

resource "aws_route_table_association" "public" {
  subnet_id      = aws_subnet.public.id
  route_table_id = aws_route_table.public.id
}

resource "aws_security_group" "web" {
  name        = "dikumud-web"
  description = "HTTP and HTTPS in to Caddy; anything out. No SSH: use SSM Session Manager."
  vpc_id      = aws_vpc.main.id
}

resource "aws_vpc_security_group_ingress_rule" "http" {
  security_group_id = aws_security_group.web.id
  description       = "HTTP (ACME challenges and redirect to HTTPS)"
  ip_protocol       = "tcp"
  from_port         = 80
  to_port           = 80
  cidr_ipv4         = "0.0.0.0/0"
}

resource "aws_vpc_security_group_ingress_rule" "https" {
  security_group_id = aws_security_group.web.id
  description       = "HTTPS"
  ip_protocol       = "tcp"
  from_port         = 443
  to_port           = 443
  cidr_ipv4         = "0.0.0.0/0"
}

resource "aws_vpc_security_group_ingress_rule" "http3" {
  security_group_id = aws_security_group.web.id
  description       = "HTTP/3"
  ip_protocol       = "udp"
  from_port         = 443
  to_port           = 443
  cidr_ipv4         = "0.0.0.0/0"
}

resource "aws_vpc_security_group_egress_rule" "all" {
  security_group_id = aws_security_group.web.id
  ip_protocol       = "-1"
  cidr_ipv4         = "0.0.0.0/0"
}

# ---- container registry ---------------------------------------------------

resource "aws_ecr_repository" "dikumud" {
  name                 = "dikumud"
  image_tag_mutability = "MUTABLE"
  image_scanning_configuration {
    scan_on_push = true
  }
}

resource "aws_ecr_lifecycle_policy" "dikumud" {
  repository = aws_ecr_repository.dikumud.name
  policy = jsonencode({
    rules = [{
      rulePriority = 1
      description  = "Keep the 10 most recent images"
      selection = {
        tagStatus   = "any"
        countType   = "imageCountMoreThan"
        countNumber = 10
      }
      action = { type = "expire" }
    }]
  })
}

# ---- instance role: pull from ECR, be reachable by SSM --------------------

data "aws_iam_policy_document" "ec2_assume" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["ec2.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "server" {
  name               = "dikumud-server"
  assume_role_policy = data.aws_iam_policy_document.ec2_assume.json
}

resource "aws_iam_role_policy_attachment" "ssm" {
  role       = aws_iam_role.server.name
  policy_arn = "arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore"
}

resource "aws_iam_role_policy_attachment" "ecr" {
  role       = aws_iam_role.server.name
  policy_arn = "arn:aws:iam::aws:policy/AmazonEC2ContainerRegistryReadOnly"
}

# The Docker daemon ships container output to CloudWatch (see daemon.json in
# cloud-init). It may only write streams in the dikumud log group.
data "aws_iam_policy_document" "server_logs" {
  statement {
    actions   = ["logs:CreateLogStream", "logs:PutLogEvents"]
    resources = ["${aws_cloudwatch_log_group.containers.arn}:*"]
  }
}

resource "aws_iam_role_policy" "server_logs" {
  name   = "cloudwatch-logs"
  role   = aws_iam_role.server.id
  policy = data.aws_iam_policy_document.server_logs.json
}

resource "aws_iam_instance_profile" "server" {
  name = "dikumud-server"
  role = aws_iam_role.server.name
}

# ---- logs -------------------------------------------------------------------

# Every container's output, one stream per container name (dikumud, caddy).
resource "aws_cloudwatch_log_group" "containers" {
  name              = "dikumud"
  retention_in_days = var.log_retention_days
}

# ---- the server -----------------------------------------------------------

data "aws_ssm_parameter" "al2023_arm64" {
  name = "/aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-arm64"
}

resource "aws_ebs_volume" "data" {
  availability_zone = local.az
  size              = var.data_volume_gb
  type              = "gp3"
  encrypted         = true
  tags              = { Name = "dikumud-data", Snapshot = "dikumud-daily" }

  lifecycle {
    prevent_destroy = true
  }
}

resource "aws_instance" "server" {
  # Pinned at creation; a newer AMI only arrives when the instance is
  # replaced for another reason (the data volume survives that).
  ami                         = nonsensitive(data.aws_ssm_parameter.al2023_arm64.value)
  instance_type               = var.instance_type
  subnet_id                   = aws_subnet.public.id
  vpc_security_group_ids      = [aws_security_group.web.id]
  iam_instance_profile        = aws_iam_instance_profile.server.name
  associate_public_ip_address = true

  user_data = templatefile("${path.module}/cloud-init.yaml.tftpl", {
    data_volume_id = replace(aws_ebs_volume.data.id, "-", "")
    domain         = var.domain
    region         = var.region
    registry       = local.registry
    # deploy.sh pushes :latest and restarts the service, which pulls it.
    image       = "${aws_ecr_repository.dikumud.repository_url}:latest"
    caddy_image = var.caddy_image
    log_group   = aws_cloudwatch_log_group.containers.name
  })
  user_data_replace_on_change = true

  metadata_options {
    http_tokens                 = "required"
    http_put_response_hop_limit = 1
  }

  root_block_device {
    volume_type = "gp3"
    volume_size = 8
    encrypted   = true
  }

  credit_specification {
    cpu_credits = "standard"
  }

  tags = { Name = "dikumud" }

  # Docker refuses to start a container whose log driver can't write, so
  # the role must be able to reach the log group before the first boot.
  depends_on = [aws_iam_role_policy.server_logs]

  lifecycle {
    ignore_changes = [ami]
  }
}

resource "aws_volume_attachment" "data" {
  device_name = "/dev/sdf"
  volume_id   = aws_ebs_volume.data.id
  instance_id = aws_instance.server.id
}

resource "aws_eip" "server" {
  domain = "vpc"
  tags   = { Name = "dikumud" }
}

resource "aws_eip_association" "server" {
  allocation_id = aws_eip.server.id
  instance_id   = aws_instance.server.id
}

# ---- DNS --------------------------------------------------------------------

data "aws_route53_zone" "site" {
  name         = var.domain
  private_zone = false
}

resource "aws_route53_record" "apex" {
  zone_id = data.aws_route53_zone.site.zone_id
  name    = var.domain
  type    = "A"
  ttl     = 300
  records = [aws_eip.server.public_ip]
}

resource "aws_route53_record" "www" {
  zone_id = data.aws_route53_zone.site.zone_id
  name    = "www.${var.domain}"
  type    = "A"
  ttl     = 300
  records = [aws_eip.server.public_ip]
}

# Only Let's Encrypt may issue certificates for the domain.
resource "aws_route53_record" "caa" {
  zone_id = data.aws_route53_zone.site.zone_id
  name    = var.domain
  type    = "CAA"
  ttl     = 3600
  records = ["0 issue \"letsencrypt.org\""]
}

# ---- daily snapshots of the data volume -------------------------------------

data "aws_iam_policy_document" "dlm_assume" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["dlm.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "dlm" {
  name               = "dikumud-dlm"
  assume_role_policy = data.aws_iam_policy_document.dlm_assume.json
}

resource "aws_iam_role_policy_attachment" "dlm" {
  role       = aws_iam_role.dlm.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSDataLifecycleManagerServiceRole"
}

resource "aws_dlm_lifecycle_policy" "data" {
  description        = "dikumud data volume daily snapshots"
  execution_role_arn = aws_iam_role.dlm.arn
  state              = "ENABLED"

  policy_details {
    resource_types = ["VOLUME"]
    target_tags    = { Snapshot = "dikumud-daily" }

    schedule {
      name      = "daily"
      copy_tags = true
      create_rule {
        interval      = 24
        interval_unit = "HOURS"
        times         = ["09:00"]
      }
      retain_rule {
        count = var.snapshot_retain_days
      }
    }
  }
}
