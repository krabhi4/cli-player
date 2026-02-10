"""Equalizer UI widget — visual EQ with sliders and preset selector."""

from textual.app import ComposeResult
from textual.containers import Horizontal, Vertical
from textual.message import Message
from textual.reactive import reactive
from textual.widget import Widget
from textual.widgets import Button, Label, Select, Static

from ..equalizer import Equalizer, EQ_BAND_LABELS, GAIN_MIN, GAIN_MAX


class EQBandChanged(Message):
    """Band gain was changed."""

    def __init__(self, band: int, gain: float):
        super().__init__()
        self.band = band
        self.gain = gain


class EQPresetChanged(Message):
    """Preset was selected."""

    def __init__(self, preset_name: str):
        super().__init__()
        self.preset_name = preset_name


class EQBand(Widget):
    """A single EQ band with a vertical text-based slider."""

    DEFAULT_CSS = """
    EQBand {
        width: 5;
        height: 100%;
        align: center middle;
    }

    EQBand .band-label {
        height: 1;
        text-align: center;
        color: $text-muted;
        width: 5;
    }

    EQBand .band-value {
        height: 1;
        text-align: center;
        color: $accent;
        width: 5;
    }

    EQBand .band-slider {
        height: 1fr;
        text-align: center;
        width: 5;
        content-align: center middle;
    }
    """

    gain: reactive[float] = reactive(0.0)

    def __init__(self, band_index: int, label: str, **kwargs):
        super().__init__(**kwargs)
        self.band_index = band_index
        self.label = label

    def compose(self) -> ComposeResult:
        yield Static(f"{self.gain:+.0f}", classes="band-value", id=f"val-{self.band_index}")
        yield Static(self._render_slider(), classes="band-slider", id=f"slider-{self.band_index}")
        yield Static(self.label, classes="band-label")

    def watch_gain(self, value: float) -> None:
        try:
            val_w = self.query_one(f"#val-{self.band_index}", Static)
            val_w.update(f"{value:+.0f}")
            slider_w = self.query_one(f"#slider-{self.band_index}", Static)
            slider_w.update(self._render_slider())
        except Exception:
            pass

    def _render_slider(self) -> str:
        """Render a vertical-ish text slider."""
        total_steps = 12
        # Map gain from GAIN_MIN..GAIN_MAX to 0..total_steps
        ratio = (self.gain - GAIN_MIN) / (GAIN_MAX - GAIN_MIN)
        pos = int(ratio * total_steps)
        pos = max(0, min(total_steps, pos))

        # Build a simple bar representation
        bar = ""
        for i in range(total_steps, -1, -1):
            if i == pos:
                bar += "●"
            elif i == total_steps // 2:
                bar += "┼"
            else:
                bar += "│"
        return bar


class EqualizerWidget(Widget):
    """Full equalizer panel with bands and preset selector."""

    DEFAULT_CSS = """
    EqualizerWidget {
        width: 1fr;
        height: auto;
        min-height: 12;
        max-height: 20;
        background: $surface;
        border: solid $primary;
        padding: 0 1;
        display: none;
    }

    EqualizerWidget.visible {
        display: block;
    }

    EqualizerWidget .eq-header {
        height: 1;
        text-style: bold;
        color: $text;
        background: $primary;
        text-align: center;
        padding: 0 1;
    }

    EqualizerWidget .eq-controls {
        height: 2;
        align: center middle;
        padding: 0 1;
    }

    EqualizerWidget .eq-bands {
        height: 1fr;
        min-height: 6;
    }

    EqualizerWidget .eq-preset-label {
        width: auto;
        padding: 0 1;
        color: $text;
    }

    EqualizerWidget Button {
        min-width: 8;
        margin: 0 1;
    }
    """

    def __init__(self, equalizer: Equalizer, **kwargs):
        super().__init__(**kwargs)
        self.equalizer = equalizer

    def compose(self) -> ComposeResult:
        yield Static("🎛 Equalizer", classes="eq-header")
        with Horizontal(classes="eq-controls"):
            presets = [(p.name, p.name) for p in self.equalizer.get_presets()]
            yield Static("Preset: ", classes="eq-preset-label")
            yield Select(
                presets,
                value=self.equalizer.get_current_preset_name(),
                id="eq-preset-select",
                allow_blank=False,
            )
            yield Button("Reset", id="eq-reset", variant="warning")
            yield Button("On/Off", id="eq-toggle", variant="primary")
        with Horizontal(classes="eq-bands"):
            for i, label in enumerate(EQ_BAND_LABELS):
                band = EQBand(i, label, id=f"eq-band-{i}")
                band.gain = self.equalizer.gains[i]
                yield band

    def on_select_changed(self, event: Select.Changed) -> None:
        if event.select.id == "eq-preset-select" and event.value:
            self.equalizer.load_preset(str(event.value))
            self._sync_bands()
            self.post_message(EQPresetChanged(str(event.value)))

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "eq-reset":
            self.equalizer.reset()
            self._sync_bands()
        elif event.button.id == "eq-toggle":
            self.equalizer.toggle()

    def _sync_bands(self):
        """Sync band widgets with equalizer state."""
        for i in range(18):
            try:
                band = self.query_one(f"#eq-band-{i}", EQBand)
                band.gain = self.equalizer.gains[i]
            except Exception:
                pass

    def adjust_band(self, band_index: int, delta: float):
        """Adjust a band's gain by delta."""
        new_gain = self.equalizer.gains[band_index] + delta
        self.equalizer.set_band(band_index, new_gain)
        try:
            band = self.query_one(f"#eq-band-{band_index}", EQBand)
            band.gain = self.equalizer.gains[band_index]
        except Exception:
            pass

    def toggle_visibility(self):
        """Toggle the EQ panel visibility."""
        self.toggle_class("visible")
