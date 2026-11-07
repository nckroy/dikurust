terraform {
  required_version = "~> 1.12.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.0"
    }
  }

  # The state bucket is created by ./bootstrap (see README.md).
  backend "s3" {
    bucket       = "dikumud-tofu-state-767398022292"
    key          = "dikumud/terraform.tfstate"
    region       = "us-east-2"
    encrypt      = true
    use_lockfile = true
  }
}

provider "aws" {
  region              = var.region
  allowed_account_ids = [var.account_id]

  default_tags {
    tags = {
      Project   = "dikumud"
      ManagedBy = "opentofu"
      Repo      = "dikurust/deploy/aws"
    }
  }
}
