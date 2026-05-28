package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

const contentRootRel = "content"

// defaultContentBundleRel is the default content dir relative to the project/install root (see resolveProjectRoot).
const defaultContentBundleRel = "content/kometa"

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

// discoverContentBundles returns project-relative paths (e.g. content/kometa) for each
// subdirectory of content/ that contains a parseable site.json with site_id and output_folder.
func discoverContentBundles(projectRoot string) ([]string, error) {
	contentDir := filepath.Join(projectRoot, contentRootRel)
	entries, err := os.ReadDir(contentDir)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, nil
		}
		return nil, fmt.Errorf("cannot read %q: %w", contentDir, err)
	}

	var bundles []string
	for _, ent := range entries {
		if !ent.IsDir() {
			continue
		}
		rel := filepath.ToSlash(filepath.Join(contentRootRel, ent.Name()))
		siteJSON := filepath.Join(projectRoot, filepath.FromSlash(rel), "site.json")
		if isValidSiteJSON(siteJSON) {
			bundles = append(bundles, rel)
		}
	}
	sort.Strings(bundles)
	return bundles, nil
}

func isValidSiteJSON(path string) bool {
	st, err := os.Stat(path)
	if err != nil || st.IsDir() {
		return false
	}
	data, err := os.ReadFile(path)
	if err != nil {
		return false
	}
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(data, &raw); err != nil {
		return false
	}
	_, hasID := raw["site_id"]
	_, hasOut := raw["output_folder"]
	return hasID && hasOut
}

func resolveSiteDir(projectRoot, siteInput string) string {
	siteInput = strings.TrimSpace(siteInput)
	if siteInput == "" {
		return filepath.Join(projectRoot, filepath.FromSlash(defaultContentBundleRel))
	}
	if filepath.IsAbs(siteInput) {
		return siteInput
	}
	return filepath.Join(projectRoot, filepath.FromSlash(siteInput))
}
