terraform {
  required_version = "~> 1.3"

  backend "local" {
    path = "states/terraform.tfstate"
  }

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 4.0"
    }
  }
}
