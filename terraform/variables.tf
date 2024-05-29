variable "region" {}

variable "project_name" {
  description = "Name of the project"
  type        = string
  default     = "cms-lambda"
}

variable "aws_access_key_id" {
  description = "The AWS access key ID"
  type        = string
  default     = ""
}

variable "aws_secret_access_key" {
  description = "The AWS secret access key"
  type        = string
  default     = ""
}

variable "lambda_functions" {
  default = {
    function1 = {
      name     = "lambda-function-1"
      handler  = "bootstrap"
      runtime  = "provided.al2"
      filename = "./lambda1.zip"
    }

    function2 = {
      name     = "lambda-function-2"
      handler  = "bootstrap"
      runtime  = "provided.al2"
      filename = "./lambda2.zip"
    }
  }
}
