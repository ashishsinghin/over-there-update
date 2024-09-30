To download a file using `curl` and ensure it has the same name as it is stored on the server, you can follow these steps:

1. **Make Sure the Server is Set Up Correctly**: Ensure your server is serving the file with the correct `Content-Disposition` header as outlined in your `downloadNewVersion` function.

2. **Use the Following Command**:

   If the `Content-Disposition` header is set correctly, you can use `curl` with the `-O` option:

   ```bash
   curl -O "http://localhost:8080/download?version=1.2.0"
   ```

   This will attempt to download the file using the name provided in the URL. However, if you want to ensure the filename comes from the `Content-Disposition` header, you can use the following method:

3. **Fetch Headers and Download**:

   First, you can fetch the headers to see the filename suggested by the server:

   ```bash
   curl -I "http://localhost:8080/download?version=1.2.0"
   ```

   Look for the `Content-Disposition` header in the response.

4. **Download with Explicit Filename**:

   If you see the correct filename in the headers, download it using:

   ```bash
   curl -o "app_1.2.0.zip" "http://localhost:8080/download?version=1.2.0"
   ```

   Replace `"app_1.2.0.zip"` with whatever filename you see in the `Content-Disposition` header.

5. **Automate Extraction of Filename** (Optional):

   If you want to automate the process, you can run the following script:

   ```bash
   # Fetch the headers and extract the filename
   filename=$(curl -s -D - "http://localhost:8080/download?version=1.2.0" -o /dev/null | grep -i "^Content-Disposition:" | sed -n 's/.*filename="\([^"]*\)".*/\1/p')

   # Download the file using the extracted filename
   curl -o "$filename" "http://localhost:8080/download?version=1.2.0"
   ```

This script will automatically extract the filename from the headers and download the file with that name.