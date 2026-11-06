"""Compile external consumers without this package's dev-dependency feature unification."""

import json
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib


def main():
    repository = Path(__file__).resolve().parents[2]
    package = tomllib.loads((repository / "Cargo.toml").read_text(encoding="utf-8"))
    eframe = package["dev-dependencies"]["eframe"]
    eframe_version = eframe["version"] if isinstance(eframe, dict) else eframe
    base = '[package]\nname = "spaceui-consumer"\nversion = "0.0.0"\nedition = "2024"\n\n[workspace]\n\n[dependencies]\n'
    core = '''use spaceui::{egui, Theme};
fn main() {
    let context = egui::Context::default();
    Theme::carbon_copper().install(&context);
    context.run_ui(Default::default(), |ui| {
        let _ = spaceui::widgets::primary(ui, "Consumer");
        let _ = ui.add(spaceui::widgets::Button::primary("Configured")
            .min_size(egui::vec2(120.0, 40.0)));
        let _ = spaceui::widgets::tabs_response(ui, &mut 0, &["One", "Two"]);
        let _ = spaceui::theme::ControlMetrics::of(ui.ctx());
    }).drop_without_applying_deltas();
}
'''
    cases = [
        ("core", False, core, ""),
        ("desktop", True, (repository / "examples/minimal.rs").read_text(encoding="utf-8"),
         f'eframe = {{ version = {json.dumps(eframe_version)}, default-features = false, features = ["glow", "default_fonts", "wayland", "x11"] }}\n'),
    ]
    environment = os.environ.copy()
    environment["CARGO_TARGET_DIR"] = str(repository / "target" / "consumer")
    with tempfile.TemporaryDirectory(prefix="spaceui-consumers-") as folder:
        for name, defaults, source, extra in cases:
            consumer = Path(folder) / name
            (consumer / "src").mkdir(parents=True)
            dependency = f'spaceui = {{ path = {json.dumps(repository.as_posix())}, default-features = {str(defaults).lower()} }}\n'
            (consumer / "Cargo.toml").write_text(base + dependency + extra, encoding="utf-8")
            (consumer / "src/main.rs").write_text(source, encoding="utf-8")
            # Resolve this consumer's graph; do not copy the development lockfile.
            subprocess.run(["cargo", "check", "--manifest-path", str(consumer / "Cargo.toml")], env=environment, check=True)
            if name == "core":
                subprocess.run(["cargo", "run", "--locked", "--manifest-path", str(consumer / "Cargo.toml")], env=environment, check=True)


if __name__ == "__main__":
    main()
