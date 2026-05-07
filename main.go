package main

import (
	"bufio"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
)

func main() {
	reader := bufio.NewReader(os.Stdin)
	rootDir, err := os.Getwd()
	must(err)
	templateDir := filepath.Join(rootDir, "Template")

	fmt.Print("Enter site bundle directory path (absolute or relative; empty = ./sites/kometa): ")
	siteInput, err := readOptionalLine(reader)
	must(err)
	siteDir := filepath.Join(rootDir, "sites", "kometa")
	if siteInput != "" {
		if filepath.IsAbs(siteInput) {
			siteDir = siteInput
		} else {
			siteDir = filepath.Join(rootDir, siteInput)
		}
	}

	bundle, warnings, err := loadSiteBundle(siteDir)
	must(err)
	for _, warning := range warnings {
		fmt.Fprintln(os.Stderr, "Warning:", warning.String())
	}

	config := Config{
		OutputFolder: bundle.Site.OutputFolder,
		Theme:        bundle.Site.Theme,
		Typography:   bundle.Site.Typography,
		Nav:          bundle.Site.Header.Nav,
		Social:       bundle.Site.Social,
		Footer:       bundle.Site.Footer,
	}

	outputFolder, err := validatedOutputFolder(config.OutputFolder)
	must(err)

	fmt.Printf("Enter destination directory (absolute or relative path, empty = project root).\n(Please note that website will be created in subdirectory: %q)\n", outputFolder)
	baseLocation, err := readOptionalLine(reader)
	must(err)
	if baseLocation == "" {
		baseLocation = rootDir
	}

	targetDir := filepath.Join(baseLocation, outputFolder)
	must(prepareDestination(targetDir))

	must(copyDir(templateDir, targetDir))
	placeholderValues := buildPlaceholders(rootDir, config)
	must(applyConfigToDir(targetDir, placeholderValues))
	must(copyConfigAssets(rootDir, targetDir, config))

	fmt.Println("Website generated successfully.")
	fmt.Println("Output:", targetDir)
}

func readOptionalLine(reader *bufio.Reader) (string, error) {
	line, err := reader.ReadString('\n')
	if err != nil && !errors.Is(err, io.EOF) {
		return "", err
	}
	return strings.TrimSpace(line), nil
}

func must(err error) {
	if err != nil {
		fmt.Fprintln(os.Stderr, "Error:", err)
		os.Exit(1)
	}
}
