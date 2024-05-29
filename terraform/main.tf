provider "aws" {
  region  = var.region
  profile = "default"
}

resource "aws_s3_bucket" "lambda_bucket" {
  bucket = local.bucket_name
}
resource "aws_iam_role" "lambda_exec" {
  name = "serverless_example_lambda"

  assume_role_policy = <<EOF
  {
    "Version": "2012-10-17",
    "Statement": [
      {
        "Action": "sts:AssumeRole",
        "Principal": {
          "Service": "lambda.amazonaws.com"
        },
        "Effect": "Allow",
        "Sid": ""
      }
    ]
  }
  EOF
}

resource "aws_iam_role_policy" "lambda_policy" {
  name = "lambda-s3-policy"
  role = aws_iam_role.lambda_exec.id

  policy = jsondecode({
    Version = "2012-10"
    Statement = [{
      Effect = "Allow"
      Action = [
        "s3:PutObject",
        "s3:GetObject",
        "s3:DeleteObject",
        "s3:ListBucket"
      ]
      Resource = [
        aws_s3_bucket.lambda_bucket.arn,
        "${aws_s3_bucket.lambda_bucket.arn}/*"
      ]
    }]
  })
}

resource "aws_lambda_function" "rust_lambda" {
  for_each = var.lambda_functions

  function_name = each.value.name
  handler       = each.value.handler
  runtime       = each.value.runtime
  role          = aws_iam_role.lambda_exec.arn
  filename      = each.value.filename

  source_code_hash = filebase64sha256(each.value.filename)
  publish          = true

  environment {
    variables = {
      BUCKET_NAME           = local.bucket_name
      RUST_LOG              = "debug"
      AWS_REGION            = var.region
      AWS_ACCESS_KEY_ID     = var.aws_access_key_id
      AWS_SECRET_ACCESS_KEY = var.aws_secret_access_key
    }
  }
}

resource "aws_api_gateway_rest_api" "apigw" {
  name        = "serverless_lambda"
  description = "Terraform Serverless Lambda Application"
}

resource "aws_api_gateway_resource" "proxy" {
  rest_api_id = aws_api_gateway_rest_api.apigw.id
  parent_id   = aws_api_gateway_rest_api.apigw.root_resource_id
  path_part   = "{proxy+}"
}

resource "aws_api_gateway_method" "proxy" {
  rest_api_id   = aws_api_gateway_rest_api.apigw.id
  resource_id   = aws_api_gateway_resource.proxy.id
  http_method   = "ANY"
  authorization = "NONE"
}

resource "aws_api_gateway_integration" "lambda" {
  count       = length(var.lambda_functions)
  rest_api_id = aws_api_gateway_rest_api.apigw.id
  resource_id = aws_api_gateway_method.proxy.resource_id
  http_method = aws_api_gateway_method.proxy.http_method

  integration_http_method = "POST"
  type                    = "AWS_PROXY"
  uri                     = aws_lambda_function.rust_lambda[count.index].invoke_arn
}

locals {
  methods_types = ["GET", "PUT", "DELETE"]
}

resource "aws_api_gateway_method" "lambda1_methods" {
  for_each    = { for method in local.methods_types : method => method }
  rest_api_id = aws_api_gateway_rest_api.apigw.id
  resource_id = aws_api_gateway_resource.proxy.id
  http_method = each.value

  authorization = "NONE"
}

resource "aws_api_gateway_integration" "lambda1_integration" {
  for_each                = aws_api_gateway_method.lambda1_methods
  rest_api_id             = aws_api_gateway_rest_api.apigw.id
  resource_id             = aws_api_gateway_resource.proxy.id
  http_method             = each.value
  type                    = "AWS_PROXY"
  integration_http_method = each.value
  uri                     = aws_lambda_function.rust_lambda[0].invoke_arn
}

resource "aws_api_gateway_method" "lambda2_methods" {
  for_each    = { for method in local.methods_types : method => method }
  rest_api_id = aws_api_gateway_rest_api.apigw.id
  resource_id = aws_api_gateway_resource.proxy.id
  http_method = each.value

  authorization = "NONE"
}

resource "aws_api_gateway_integration" "lambda2_integration" {
  for_each                = aws_api_gateway_method.lambda2_methods
  rest_api_id             = aws_api_gateway_rest_api.apigw.id
  resource_id             = aws_api_gateway_resource.proxy.id
  http_method             = each.value
  type                    = "AWS_PROXY"
  integration_http_method = each.value
  uri                     = aws_lambda_function.rust_lambda[1].invoke_arn
}

resource "aws_api_gateway_deployment" "apigw_deploy" {
  depends_on = [
    aws_api_gateway_integration.lambda,
    aws_api_gateway_integration.lambda1_integration,
    aws_api_gateway_integration.lambda2_integration
  ]

  rest_api_id = aws_api_gateway_rest_api.apigw.id
  stage_name  = "test"
}

resource "aws_lambda_permission" "apigw_permission_lambda1" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.rust_lambda[0].function_name
  principal     = "apigateway.amazonaws.com"

  source_arn = "${aws_api_gateway_rest_api.apigw.execution_arn}/*/ANY/lambda1"
}

resource "aws_lambda_permission" "apigw_permission_lambda2" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.rust_lambda[1].function_name
  principal     = "apigateway.amazonaws.com"

  source_arn = "${aws_api_gateway_rest_api.apigw.execution_arn}/*/ANY/lambda2"
}
