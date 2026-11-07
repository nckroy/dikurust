output "url" {
  value = "https://${var.domain}/"
}

output "public_ip" {
  value = aws_eip.server.public_ip
}

output "instance_id" {
  description = "For `aws ssm start-session --target <id>`."
  value       = aws_instance.server.id
}

output "ecr_repository_url" {
  value = aws_ecr_repository.dikumud.repository_url
}

output "log_group" {
  description = "For `aws logs tail <group> --follow`."
  value       = aws_cloudwatch_log_group.containers.name
}
