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

// defaultContentBundleRel is the default content dir relative to the project/install root (see resolveProjectRoot).
const defaultContentBundleRel = "content/kometa"

func main() {
	reader := bufio.NewReader(os.Stdin)
	projectRoot, err := resolveProjectRoot()
	must(err)
	templateDir := filepath.Join(projectRoot, "Template")

	fmt.Print("Enter content bundle directory path (absolute or relative; empty = ./content/kometa): ")
	siteInput, err := readOptionalLine(reader)
	must(err)
	siteDir := filepath.Join(projectRoot, filepath.FromSlash(defaultContentBundleRel))
	if siteInput != "" {
		if filepath.IsAbs(siteInput) {
			siteDir = siteInput
		} else {
			siteDir = filepath.Join(projectRoot, filepath.FromSlash(siteInput))
		}
	}

	bundle, warnings, err := loadSiteBundle(siteDir)
	must(err)
	for _, warning := range warnings {
		fmt.Fprintln(os.Stderr, "Warning:", warning.String())
	}

	outputFolder, err := validatedOutputFolderFor(bundle.Site.OutputFolder, bundle.SitePath)
	must(err)

	targetDir := filepath.Join(projectRoot, filepath.FromSlash(outputFolder))
	must(prepareDestination(targetDir))

	must(copyTemplateStaticAssets(templateDir, targetDir))
	must(copyReferencedSiteAssets(bundle, targetDir))
	must(renderSiteBundle(bundle, targetDir, templateDir))

	fmt.Println("Website generated successfully.")
	fmt.Println("Output:", targetDir)
}

// resolveProjectRoot picks the directory that contains Template/ and the default content bundle when the
// user runs the binary from another working directory (e.g. ~/ with the executable in the repo root).
// Order: cwd if it contains content/kometa/site.json, else executable directory if it does, else cwd.
func resolveProjectRoot() (string, error) {
	wd, errWd := os.Getwd()
	if errWd != nil {
		wd = ""
	}
	if wd != "" && contentBundleMarkerExists(wd) {
		return filepath.Clean(wd), nil
	}
	exe, err := os.Executable()
	if err != nil {
		if wd != "" {
			return filepath.Clean(wd), nil
		}
		return "", err
	}
	exe, err = filepath.EvalSymlinks(exe)
	if err != nil {
		return "", err
	}
	exeDir := filepath.Dir(exe)
	if contentBundleMarkerExists(exeDir) {
		return filepath.Clean(exeDir), nil
	}
	if wd != "" {
		return filepath.Clean(wd), nil
	}
	return filepath.Clean(exeDir), nil
}

func contentBundleMarkerExists(root string) bool {
	st, err := os.Stat(filepath.Join(root, filepath.FromSlash(defaultContentBundleRel), "site.json"))
	return err == nil && !st.IsDir()
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
