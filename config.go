package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"strings"
)

func loadConfig(path string) (Config, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return Config{}, fmt.Errorf("cannot read config %q: %w", path, err)
	}

	var config Config
	if err := json.Unmarshal(data, &config); err != nil {
		return Config{}, fmt.Errorf("invalid config json %q: %w", path, err)
	}
	return config, nil
}

func validatedOutputFolder(name string) (string, error) {
	s := strings.TrimSpace(name)
	if s == "" {
		return "", errors.New(`config.json: "output_folder" is required and must not be empty`)
	}
	if strings.ContainsAny(s, `/\`) {
		return "", fmt.Errorf(`config.json: "output_folder" must not contain path separators (got %q)`, s)
	}
	if s == "." || s == ".." || strings.Contains(s, "..") {
		return "", fmt.Errorf(`config.json: invalid "output_folder" %q`, s)
	}
	return s, nil
}
