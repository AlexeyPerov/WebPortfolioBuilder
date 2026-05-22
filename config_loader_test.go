package main

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestValidatedOutputFolderAllowsNestedRelativePath(t *testing.T) {
	got, err := validatedOutputFolderFor("Results/KometaWebsite", "site.json")
	if err != nil {
		t.Fatalf("expected valid nested path, got error: %v", err)
	}
	if got != "Results/KometaWebsite" {
		t.Fatalf("expected normalized path, got %q", got)
	}
}

func TestValidatedOutputFolderRejectsAbsolutePath(t *testing.T) {
	_, err := validatedOutputFolderFor("/tmp/out", "site.json")
	if err == nil {
		t.Fatal("expected absolute path error")
	}
}

func TestLoadSiteBundleRejectsDuplicateSlugs(t *testing.T) {
	siteDir := createTestSiteDir(t)
	writeJSONFile(t, filepath.Join(siteDir, "site.json"), `{"site_id":"kometa","output_folder":"KometaWebsite"}`)
	writeJSONFile(t, filepath.Join(siteDir, "pages", "home.json"), `{"slug":"","widgets":[]}`)
	writeJSONFile(t, filepath.Join(siteDir, "pages", "about.json"), `{"slug":"","widgets":[]}`)

	_, _, err := loadSiteBundle(siteDir)
	if err == nil {
		t.Fatal("expected duplicate slug error")
	}
}

func TestLoadSiteBundleRejectsInvalidOutputFolder(t *testing.T) {
	siteDir := createTestSiteDir(t)
	writeJSONFile(t, filepath.Join(siteDir, "site.json"), `{"site_id":"kometa","output_folder":"../bad"}`)
	writeJSONFile(t, filepath.Join(siteDir, "pages", "home.json"), `{"slug":"","widgets":[]}`)

	_, _, err := loadSiteBundle(siteDir)
	if err == nil {
		t.Fatal("expected invalid output_folder error")
	}
}

func TestLoadSiteBundleWarnsOnUnknownTopLevelKeys(t *testing.T) {
	siteDir := createTestSiteDir(t)
	writeJSONFile(t, filepath.Join(siteDir, "site.json"), `{"site_id":"kometa","output_folder":"KometaWebsite","mystery":"x"}`)
	writeJSONFile(t, filepath.Join(siteDir, "pages", "home.json"), `{"slug":"","widgets":[],"unused_flag":true}`)

	_, warnings, err := loadSiteBundle(siteDir)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if len(warnings) != 2 {
		t.Fatalf("expected 2 warnings, got %d", len(warnings))
	}
}

func TestLoadSiteBundleRejectsMissingWidgetsField(t *testing.T) {
	siteDir := createTestSiteDir(t)
	writeJSONFile(t, filepath.Join(siteDir, "site.json"), `{"site_id":"kometa","output_folder":"KometaWebsite"}`)
	writeJSONFile(t, filepath.Join(siteDir, "pages", "home.json"), `{"slug":""}`)

	_, _, err := loadSiteBundle(siteDir)
	if err == nil {
		t.Fatal("expected widgets required error")
	}
}

func TestLoadSiteBundleRejectsDuplicateProjectGridSectionIDs(t *testing.T) {
	siteDir := createTestSiteDir(t)
	writeJSONFile(t, filepath.Join(siteDir, "site.json"), `{"site_id":"kometa","output_folder":"KometaWebsite"}`)
	writeJSONFile(t, filepath.Join(siteDir, "pages", "home.json"), `{"slug":"","widgets":[
		{"type":"project_grid","props":{"section_id":"box","cards":[]}},
		{"type":"project_grid","props":{"section_id":"box","cards":[]}}
	]}`)

	_, _, err := loadSiteBundle(siteDir)
	if err == nil {
		t.Fatal("expected duplicate section_id error")
	}
	if !strings.Contains(err.Error(), "duplicate project_grid") {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestLoadSiteBundleRejectsNestedDuplicateProjectGridSectionIDs(t *testing.T) {
	siteDir := createTestSiteDir(t)
	writeJSONFile(t, filepath.Join(siteDir, "site.json"), `{"site_id":"kometa","output_folder":"KometaWebsite"}`)
	writeJSONFile(t, filepath.Join(siteDir, "pages", "home.json"), `{"slug":"","widgets":[
		{"type":"row","props":{"children":[
			{"type":"project_grid","props":{"section_id":"same","cards":[]}},
			{"type":"project_grid","props":{"section_id":"same","cards":[]}}
		]}}
	]}`)

	_, _, err := loadSiteBundle(siteDir)
	if err == nil {
		t.Fatal("expected nested duplicate section_id error")
	}
}

func TestKometaSiteBundleLoads(t *testing.T) {
	_, warnings, err := loadSiteBundle(filepath.Join("content", "kometa"))
	if err != nil {
		t.Fatalf("kometa bundle should load: %v", err)
	}
	if len(warnings) != 0 {
		t.Fatalf("unexpected warnings: %v", warnings)
	}
}

func createTestSiteDir(t *testing.T) string {
	t.Helper()

	root := t.TempDir()
	siteDir := filepath.Join(root, "content", "test-site")
	pagesDir := filepath.Join(siteDir, "pages")
	if err := os.MkdirAll(pagesDir, 0o755); err != nil {
		t.Fatalf("cannot create test directories: %v", err)
	}
	return siteDir
}

func writeJSONFile(t *testing.T, path, content string) {
	t.Helper()

	if err := os.WriteFile(path, []byte(content), 0o644); err != nil {
		t.Fatalf("cannot write %s: %v", path, err)
	}
}
