curl http://$1:8080/check-update-blink-five?current_version=1.0.0

# Fetch the headers and extract the filename
filenamefive=$(curl -s -D - "http://$1:8080/download-blink-five?version=1.2.0" -o /dev/null | grep -i "^Content-Disposition:" | sed -n 's/.*filename="\([^"]*\)".*/\1/p')

# Download the file using the extracted filename
curl -o "$filenamefive" "http://$1:8080/download-blink-five?version=1.2.0"