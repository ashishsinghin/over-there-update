# Fetch the headers and extract the filename
filenameone=$(curl -s -D - "http://$1:8080/download-blink-one?version=1.2.0" -o /dev/null | grep -i "^Content-Disposition:" | sed -n 's/.*filename="\([^"]*\)".*/\1/p')

# Download the file using the extracted filename
curl -o "$filenameone" "http://$1:8080/download-blink-one?version=1.2.0"
