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

	fmt.Print("Enter config.json path to use (absolute or relative; empty = ./config.json in project root): ")
	configInput, err := readOptionalLine(reader)
	must(err)
	configPath := filepath.Join(rootDir, "config.json")
	if configInput != "" {
		if filepath.IsAbs(configInput) {
			configPath = configInput
		} else {
			configPath = filepath.Join(rootDir, configInput)
		}
	}

	config, err := loadConfig(configPath)
	must(err)

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
