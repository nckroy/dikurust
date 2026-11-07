variable "account_id" {
  description = "The only AWS account this stack may touch."
  type        = string
  default     = "767398022292"
}

variable "region" {
  type    = string
  default = "us-east-2"
}

variable "domain" {
  description = "Route 53 hosted zone and site name. www.<domain> redirects to it."
  type        = string
  default     = "diku.earth"
}

variable "instance_type" {
  description = "Must be an arm64 (Graviton) type: the image is built for linux/arm64."
  type        = string
  default     = "t4g.micro"
}

variable "data_volume_gb" {
  description = "Size of the EBS volume holding player files, the board and Caddy's certificates."
  type        = number
  default     = 2
}

variable "caddy_image" {
  type    = string
  default = "caddy:2.11.9-alpine"
}

variable "snapshot_retain_days" {
  description = "Daily snapshots of the data volume to keep."
  type        = number
  default     = 14
}
