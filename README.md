# GameDevStudio Site Creator

This project is a small Go-based static site generator.

Main idea:
- Take files from `Template/`
- Read your chosen `config.json`
- Replace placeholders like `{{site_title}}` and build dynamic blocks (games, offers, vacancies, etc.)
- Copy referenced assets from `Images/` (and other configured paths)
- Create a ready-to-open website in the configured output subdirectory

## How To Run (Beginner Friendly)

If you never used Go before, follow these steps:

1. Install Go (1.20+ recommended):
   - [https://go.dev/dl/](https://go.dev/dl/)
2. Open terminal and go to project folder:
   - `cd /path/to/GameDevStudio-SiteCreator`
3. Run the generator from the project folder:
   - `go run .`
4. Answer prompts:
   - Config path prompt:
     - Press Enter to use default `./config.json`
     - Or type another config file path
   - Destination directory prompt:
     - Press Enter to use project root
     - Or provide another absolute/relative directory
5. Open generated site:
   - The program prints the final output path
   - Open generated `index.html` in browser

Notes:
- The output subdirectory name comes from `output_folder` in your config.
- If an asset path in config is wrong, generation fails with a clear error.

## Add / Remove Job Postings

Job postings are stored in `config.json` under `vacancies` array.

Add a vacancy:
- Append a new object to `vacancies` with fields:
  - `role`
  - `requirements` (array of strings)
  - `responsibilities` (array of strings)
  - `advantages` (array of strings)
  - optional: `apply_url`, `apply_label`

Remove a vacancy:
- Delete that object from `vacancies`.

Hide entire Careers section:
- Set `"vacancies": []`
- The generator will omit the whole vacancies section automatically.
