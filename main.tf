provider "aws" {
    region = "ap-south-2"
}

# Define the AWS instance resource
resource "aws_instance" "my_instance" {
    ami           = "ami-068daf89d1895ab7b"
    instance_type = "t3.micro"
    key_name      = "bosch-ec2-rpi4"

    tags = {
        Name = "ota-server"
    }
}

# Associate an Elastic IP with the instance
resource "aws_eip" "my_eip" {
    instance = aws_instance.my_instance.id
}

# Use aws_ec2_instance_state to start the instance if it's stopped
resource "aws_ec2_instance_state" "start_my_instance" {
    instance_id = aws_instance.my_instance.id
    state       = "running" // to start "running" and to stop "stopped"
}

# Data source to get instance details
data "aws_instance" "my_instance_data" {
    instance_id = aws_instance.my_instance.id
}

# Outputs
output "instance_id" {
    value = aws_instance.my_instance.id
}

output "instance_state" {
    value = data.aws_instance.my_instance_data.instance_state
}

output "elastic_ip" {
    value = aws_eip.my_eip.public_ip
}

