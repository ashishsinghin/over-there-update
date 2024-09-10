package main

import (
	"fmt"
	"net/http"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/gin-gonic/gin"
)

type VersionInfo struct {
	LatestVersion string `json:"latest_version"`
	DownloadURL   string `json:"download_url,omitempty"`
}

const otaFilesPath = "./ota_files/"

// Helper function to parse the version from the file name (e.g., "app_1.2.0.zip")
func extractVersionFromFile(fileName string) string {
	baseName := strings.TrimSuffix(fileName, filepath.Ext(fileName)) // remove extension
	parts := strings.Split(baseName, "_")
	if len(parts) == 2 {
		return parts[1] // version part
	}
	return ""
}

// Helper function to get all OTA files and extract versions
func getAvailableVersions() ([]string, error) {
	var versions []string

	// Walk through the OTA files directory
	err := filepath.Walk(otaFilesPath, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if !info.IsDir() {
			version := extractVersionFromFile(info.Name())
			if version != "" {
				versions = append(versions, version)
			}
		}
		return nil
	})

	return versions, err
}

// Endpoint to check for a new version
func checkForUpdate(c *gin.Context) {
	currentVersion := c.Query("current_version")
	if currentVersion == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "current_version is required"})
		return
	}

	versions, err := getAvailableVersions()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Could not fetch available versions"})
		return
	}

	// Sort the versions to get the latest one
	sort.Strings(versions)
	latestVersion := versions[len(versions)-1]

	if latestVersion > currentVersion {
		downloadURL := fmt.Sprintf("/download?version=%s", latestVersion)
		c.JSON(http.StatusOK, VersionInfo{
			LatestVersion: latestVersion,
			DownloadURL:   downloadURL,
		})
	} else {
		c.JSON(http.StatusOK, VersionInfo{
			LatestVersion: latestVersion,
		})
	}
}

// // Endpoint to download the new version file
// func downloadNewVersion(c *gin.Context) {
// 	requestedVersion := c.Query("version")
// 	if requestedVersion == "" {
// 		c.JSON(http.StatusBadRequest, gin.H{"error": "version is required"})
// 		return
// 	}

// 	fileName := fmt.Sprintf("app_%s.wasm", requestedVersion)
// 	filePath := filepath.Join(otaFilesPath, fileName)

// 	if _, err := os.Stat(filePath); os.IsNotExist(err) {
// 		c.JSON(http.StatusNotFound, gin.H{"error": "version not found"})
// 		return
// 	}

// 	c.File(filePath)
// }

// Endpoint to download the new version file
func downloadNewVersion(c *gin.Context) {
	requestedVersion := c.Query("version")
	if requestedVersion == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "version is required"})
		return
	}

	fileName := fmt.Sprintf("app_%s.wasm", requestedVersion)
	filePath := filepath.Join(otaFilesPath, fileName)

	if _, err := os.Stat(filePath); os.IsNotExist(err) {
		c.JSON(http.StatusNotFound, gin.H{"error": "version not found"})
		return
	}

	c.Header("Content-Disposition", fmt.Sprintf("attachment; filename=\"%s\"", fileName))
	c.File(filePath)
}

func main() {
	router := gin.Default()

	// OTA version check endpoint
	router.GET("/check-update", checkForUpdate)

	// OTA file download endpoint
	router.GET("/download", downloadNewVersion)

	router.Run(":8080")
}
