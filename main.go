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

	outputFolder, err := validatedOutputFolderFor(bundle.Site.OutputFolder, bundle.SitePath)
	must(err)

	fmt.Printf("Enter destination directory (absolute or relative path, empty = project root).\n(Please note that website will be created in subdirectory: %q)\n", outputFolder)
	baseLocation, err := readOptionalLine(reader)
	must(err)
	if baseLocation == "" {
		baseLocation = rootDir
	}

	targetDir := filepath.Join(baseLocation, outputFolder)
	must(prepareDestination(targetDir))

	must(copyTemplateStaticAssets(templateDir, targetDir))
	must(copyReferencedSiteAssets(bundle, targetDir))
	must(renderSiteBundle(bundle, targetDir, templateDir))

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
