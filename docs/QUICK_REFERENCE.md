# Terminal quick reference

Launch from the checkout with `./run-tui.sh`, or install with `./install.sh -i` and run `bi0cyph3r`.

1. On the start screen, press `s` for the workbench (`g` opens the guide, `t` opens themes). Press `i` to write a message, or `Ctrl+O` to import a UTF-8 file.
2. Press `Esc` to leave editing; `m` cycles the encoding mode.
3. Use `Tab` and `Enter` to edit passwords or split keys when required.
4. Press `Ctrl+R` to run. `d` loads the result into Decode; `s` loads it into Safety.
5. Press `Ctrl+S` to export. In the export dialog, `Tab` cycles TXT / FASTA / JSON.
6. Split-key results also need `Ctrl+K` to export both keys. Store them separately.
7. Press `?` for all shortcuts or `q` to quit.

`1`–`4` select Encode, Decode, Safety, and Plasmid. Function keys `F1`–`F4` also work during editing. The Plasmid workspace supports named payloads, decoding markers, and the existing eGFP cassette with `x`.

No browser or server is used. See the [project README](../README.md) for CLI, Docker, and data handling.

Themes: Original, Crimson, Paper (white background), Monochrome, and Amber. Use arrows or `1`–`5` to preview, Enter to apply, and Esc to cancel. The choice lasts for the session; `--theme paper` or `BIOCYPHER_THEME=paper` selects a launch default. In the workbench, `t` / `F9` opens themes. Esc leaves editing, then Esc again returns home without losing your inputs. Space on the start screen pauses the rotating helix.
