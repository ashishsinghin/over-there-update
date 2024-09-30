curl http://localhost:8080/check-update?current_version=1.0.0

# Fetch the headers and extract the filename
filename=$(curl -s -D - "http://localhost:8080/download?version=1.2.0" -o /dev/null | grep -i "^Content-Disposition:" | sed -n 's/.*filename="\([^"]*\)".*/\1/p')

# Download the file using the extracted filename
curl -o "$filename" "http://localhost:8080/download?version=1.2.0"