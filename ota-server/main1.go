package main

import (
    "encoding/json"
    "fmt"
    "log"
    "net/http"
    "os"
    "path/filepath"
    "sort"
    "strings"

    "github.com/Masterminds/semver/v3"
)

// VersionInfo holds information about a specific version of a file.
type VersionInfo struct {
    Version     *semver.Version `json:"version"`      // Parsed semantic version
    FileName    string          `json:"file_name"`    // Name of the file
    DownloadURL string          `json:"download_url"` // URL to download the file
}

// Server represents the OTA server with its configuration and available versions.
type Server struct {
    FilesDir string          // Directory where files are stored
    BaseURL  string          // Base URL for constructing download links
    Versions []*VersionInfo  // Sorted list of available versions
}

// NewServer initializes the Server by loading available versions from the FilesDir.
func NewServer(filesDir, baseURL string) (*Server, error) {
    server := &Server{
        FilesDir: filesDir,
        BaseURL:  baseURL,
    }

    err := server.loadVersions()
    if err != nil {
        return nil, err
    }

    return server, nil
}

// loadVersions scans the FilesDir, parses versioned filenames, and populates the Versions slice.
func (s *Server) loadVersions() error {
    var versions []*VersionInfo

    // Walk through the FilesDir to find files
    err := filepath.WalkDir(s.FilesDir, func(path string, d os.DirEntry, err error) error {
        if err != nil {
            return err
        }

        // Skip directories
        if d.IsDir() {
            return nil
        }

        filename := d.Name()

        // Split the filename to extract the version
        // Expected format: filename_version.extension
        parts := strings.Split(filename, "_")
        if len(parts) < 2 {
            // Filename does not match the expected format; skip
            return nil
        }

        // Extract the base (without extension) and find the last underscore
        base := strings.TrimSuffix(filename, filepath.Ext(filename))
        underscoreIdx := strings.LastIndex(base, "_")
        if underscoreIdx == -1 {
            // No underscore found; invalid format
            return nil
        }

        // The version string is the part after the last underscore
        versionStr := base[underscoreIdx+1:]
        version, err := semver.NewVersion(versionStr)
        if err != nil {
            // Invalid semantic version; skip this file
            return nil
        }

        // Construct the download URL for this file
        downloadURL := fmt.Sprintf("%s/download?file=%s", s.BaseURL, filename)

        // Create a VersionInfo instance
        vInfo := &VersionInfo{
            Version:     version,
            FileName:    filename,
            DownloadURL: downloadURL,
        }

        versions = append(versions, vInfo)

        return nil
    })

    if err != nil {
        return err
    }

    if len(versions) == 0 {
        log.Println("No valid versioned files found.")
    }

    // Sort the versions in ascending order
    sort.Slice(versions, func(i, j int) bool {
        return versions[i].Version.LessThan(versions[j].Version)
    })

    s.Versions = versions
    return nil
}

// handleCheck processes the /check endpoint to determine if a newer version is available.
func (s *Server) handleCheck(w http.ResponseWriter, r *http.Request) {
    // Expecting a GET request with a query parameter 'current_version'
    currentVersionStr := r.URL.Query().Get("current_version")
    if currentVersionStr == "" {
        http.Error(w, "Missing 'current_version' parameter", http.StatusBadRequest)
        return
    }

    // Parse the current version provided by the client
    currentVersion, err := semver.NewVersion(currentVersionStr)
    if err != nil {
        http.Error(w, "Invalid 'current_version' format", http.StatusBadRequest)
        return
    }

    // Check if any version is greater than the current version
    var latestAvailable *VersionInfo
    for _, v := range s.Versions {
        if v.Version.GreaterThan(currentVersion) {
            latestAvailable = v
        }
    }

    // Find the highest version greater than the current version
    if latestAvailable != nil {
        // There is a newer version available
        response := map[string]interface{}{
            "update_available": true,
            "latest_version":    latestAvailable.Version.String(),
            "download_url":      latestAvailable.DownloadURL,
        }
        w.Header().Set("Content-Type", "application/json")
        json.NewEncoder(w).Encode(response)
    } else {
        // No newer version available
        latestVersion := "0.0.0"
        if len(s.Versions) > 0 {
            latestVersion = s.Versions[len(s.Versions)-1].Version.String()
        }
        response := map[string]interface{}{
            "update_available": false,
            "latest_version":    latestVersion,
        }
        w.Header().Set("Content-Type", "application/json")
        json.NewEncoder(w).Encode(response)
    }
}

// handleDownload processes the /download endpoint to serve the requested file.
func (s *Server) handleDownload(w http.ResponseWriter, r *http.Request) {
    // Expecting a GET request with a query parameter 'file'
    file := r.URL.Query().Get("file")
    if file == "" {
        http.Error(w, "Missing 'file' parameter", http.StatusBadRequest)
        return
    }

    // Prevent directory traversal by ensuring the file name does not contain path separators
    if strings.Contains(file, "/") || strings.Contains(file, "\\") {
        http.Error(w, "Invalid 'file' parameter", http.StatusBadRequest)
        return
    }

    filePath := filepath.Join(s.FilesDir, file)

    // Check if the file exists and is not a directory
    fi, err := os.Stat(filePath)
    if os.IsNotExist(err) {
        http.Error(w, "File not found", http.StatusNotFound)
        return
    }
    if fi.IsDir() {
        http.Error(w, "Requested file is a directory", http.StatusBadRequest)
        return
    }

    // Serve the file for download
    http.ServeFile(w, r, filePath)
}

func main() {
    // Configuration
    filesDir := "./files"               // Directory where versioned files are stored
    serverPort := ":8080"               // Server listening port
    baseURL := "http://localhost:8080"  // Base URL for constructing download URLs

    // Initialize the server
    server, err := NewServer(filesDir, baseURL)
    if err != nil {
        log.Fatalf("Failed to initialize server: %v", err)
    }

    // Define HTTP routes
    http.HandleFunc("/check", server.handleCheck)
    http.HandleFunc("/download", server.handleDownload)

    // Start the server
    fmt.Printf("OTA Server is running at %s\n", serverPort)
    log.Fatal(http.ListenAndServe(serverPort, nil))
}
