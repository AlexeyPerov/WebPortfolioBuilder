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

## Widget behavior (`widgets`)

Optional top-level object; omitted keys keep the original defaults.

- `widgets.scroll_reveal`
  - `respect_reduced_motion` (boolean, default `true`): when `true`, scroll-reveal animations are skipped if the visitor prefers reduced motion.
  - `root_margin` (string, default `"0px 0px -5% 0px"`): passed to `IntersectionObserver`.
  - `threshold` (number, default `0.12`): observer intersection threshold.
- `widgets.game_swiper`
  - `swipe_threshold_px` (number, default `30`): minimum horizontal swipe distance on touch devices to change screenshot slides.
- `widgets.split_widget`
  - `keyboard_navigation` (boolean, default `false`): when `true`, focused vacancy tabs widget responds to Left/Right arrow keys (the widget receives `tabindex="0"`).

The generator injects these settings as JSON in the built page (`site-widgets-config`). If that block is missing (for example an old hand-edited export), scripts fall back to the same defaults as before.

## Social links (`social`)

You can keep the legacy flat shape (`github_url`, `linkedin_url`, `facebook_url`) or use an explicit ordered list:

- `social.links`: array of `{ "url", "aria_label", "icon", "icon_image" }`.
  - Each entry needs a non-empty `url` and either a built-in `icon` preset (`github`, `linkedin`, `facebook`) or `icon_image` (path under the project, copied like other assets).
  - If `links` has at least one entry, only those links are shown; legacy URL fields are ignored for rendering.

## Store buttons (`games[].store_links` and `game_store_icons`)

- Legacy fields per game (`google_play_url`, `app_store_url`, `amazon_store_url`, `galaxy_store_url`) still work unchanged when `store_links` is absent or empty.
- `games[].store_links`: ordered array of `{ "url", "aria_label", "icon", "icon_image" }`. When present and non-empty, **only** these buttons are rendered for that game.
  - Use `icon` with a key matching `game_store_icons` (for example `google_play`, `steam`) **or** set `icon_image` to a project-relative image path for a fully custom badge.

`game_store_icons` is a JSON object mapping arbitrary preset keys to image paths. Defaults are provided for `google_play`, `app_store`, `galaxy`, and `amazon`; add keys such as `"steam": "Images/steam-badge.png"` for extra presets referenced from `store_links`.
