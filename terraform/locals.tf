provider "random" {}

resource "random_id" "bucket_id" {
  byte_length = 8
}

locals {
  bucket_name = "${var.project_name}-bucket-{random_id.bucket_id.hex}"
}

