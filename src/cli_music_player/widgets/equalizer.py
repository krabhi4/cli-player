"""Equalizer UI widget — visual EQ with interactive sliders and preset selector."""

from textual.app import ComposeResult
from textual.containers import Horizontal, Vertical
from textual.events import Click
from textual.message import Message
from textual.reactive import reactive
from textual.widget import Widget
from textual.widgets import Button, Input, Label, Select, Static

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
    """A single EQ band — self-rendering, clickable, keyboard-adjustable.

    Uses render() instead of compose() so clicks go directly to this widget
    without being intercepted by child Static widgets.
    """

    can_focus = True

    DEFAULT_CSS = """
    EQBand {
        width: 5;
        height: 1fr;
    }

    EQBand:focus {
        background: #414868;
    }
    """

    gain: reactive[float] = reactive(0.0)

    def __init__(self, band_index: int, label: str, **kwargs):
        super().__init__(**kwargs)
        self.band_index = band_index
        self.band_label = label

    def render(self) -> str:
        """Render the band as value + vertical slider + frequency label."""
        h = self.size.height
        if h <= 0:
            return ""

        lines = []
        # Line 0: gain value
        lines.append(f"{self.gain:+.0f}".center(5))

        # Lines 1 to h-2: slider
        slider_h = max(h - 2, 1)
        steps = slider_h - 1
        if steps <= 0:
            steps = 1

        ratio = (self.gain - GAIN_MIN) / (GAIN_MAX - GAIN_MIN)
        pos = int(ratio * steps)
        pos = max(0, min(steps, pos))
        mid = steps // 2

        for i in range(steps, -1, -1):
            if i == pos:
                lines.append("  ●  ")
            elif i == mid:
                lines.append("  ┼  ")
            else:
                lines.append("  │  ")

        # Last line: frequency label
        lines.append(self.band_label.center(5))

        return "\n".join(lines[:h])

    def on_click(self, event: Click) -> None:
        """Handle click — focus this band and map vertical position to gain."""
        self.focus()
        h = self.size.height
        if h <= 2:
            return

        slider_start = 1   # after value line
        slider_end = h - 1  # before label line
        slider_h = slider_end - slider_start
        if slider_h <= 0:
            return

        y = event.y
        if y < slider_start:
            new_gain = min(GAIN_MAX, self.gain + 1.0)
        elif y >= slider_end:
            new_gain = max(GAIN_MIN, self.gain - 1.0)
        else:
            click_y = y - slider_start
            ratio = 1.0 - (click_y / max(slider_h - 1, 1))
            new_gain = GAIN_MIN + ratio * (GAIN_MAX - GAIN_MIN)
            new_gain = max(GAIN_MIN, min(GAIN_MAX, round(new_gain)))

        # Direct update — visual + backend (bypass message system)
        self.gain = new_gain
        try:
            eq_widget = self.screen.query_one("#eq-widget", EqualizerWidget)
            eq_widget.equalizer.set_band(self.band_index, new_gain)
        except Exception:
            pass

    def _adjust_gain(self, delta: float) -> None:
        """Directly adjust gain — updates visual, backend, and audio."""
        new_gain = max(GAIN_MIN, min(GAIN_MAX, self.gain + delta))
        self.gain = new_gain  # Visual update via watch_gain
        # Directly update the equalizer backend (bypass message system)
        try:
            eq_widget = self.screen.query_one("#eq-widget", EqualizerWidget)
            eq_widget.equalizer.set_band(self.band_index, new_gain)
        except Exception:
            pass

    def on_key(self, event) -> None:
        """Handle keyboard: up/down adjust gain, left/right switch bands."""
        if event.key == "up":
            event.stop()
            event.prevent_default()
            self._adjust_gain(1.0)
        elif event.key == "down":
            event.stop()
            event.prevent_default()
            self._adjust_gain(-1.0)
        elif event.key == "left":
            event.stop()
            event.prevent_default()
            if self.band_index > 0:
                try:
                    prev_band = self.screen.query_one(f"#eq-band-{self.band_index - 1}", EQBand)
                    prev_band.focus()
                except Exception:
                    pass
        elif event.key == "right":
            event.stop()
            event.prevent_default()
            if self.band_index < 17:
                try:
                    next_band = self.screen.query_one(f"#eq-band-{self.band_index + 1}", EQBand)
                    next_band.focus()
                except Exception:
                    pass
        elif event.key == "escape":
            event.stop()
            event.prevent_default()
            try:
                eq_widget = self.screen.query_one("#eq-widget", EqualizerWidget)
                eq_widget.toggle_visibility()
            except Exception:
                pass

    def watch_gain(self, value: float) -> None:
        """Re-render when gain changes."""
        self.refresh()


class EqualizerWidget(Widget):
    """Full equalizer panel with bands and preset selector."""

    DEFAULT_CSS = """
    EqualizerWidget {
        width: 1fr;
        height: auto;
        min-height: 14;
        max-height: 24;
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
        height: 3;
        align: center middle;
        padding: 0 1;
    }

    EqualizerWidget .eq-bands {
        height: 1fr;
        min-height: 8;
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

    EqualizerWidget .eq-save-row {
        height: 3;
        align: center middle;
        padding: 0 1;
    }

    EqualizerWidget #eq-save-name {
        width: 20;
        margin: 0 1;
    }
    """

    def __init__(self, equalizer: Equalizer, **kwargs):
        super().__init__(**kwargs)
        self.equalizer = equalizer

    def compose(self) -> ComposeResult:
        yield Static("Equalizer  [e: close | ←/→: bands | ↑/↓: adjust | click: set]", classes="eq-header")
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
        with Horizontal(classes="eq-save-row"):
            yield Input(placeholder="Preset name", id="eq-save-name")
            yield Button("Save Preset", id="eq-save-preset", variant="success")

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
        elif event.button.id == "eq-save-preset":
            self._save_custom_preset()

    def on_eq_band_changed(self, event: EQBandChanged) -> None:
        """Handle band gain change from click or keyboard."""
        self.equalizer.set_band(event.band, event.gain)
        try:
            band = self.query_one(f"#eq-band-{event.band}", EQBand)
            band.gain = self.equalizer.gains[event.band]
        except Exception:
            pass

    def _save_custom_preset(self):
        """Save current EQ gains as a custom preset."""
        try:
            name_input = self.query_one("#eq-save-name", Input)
            name = name_input.value.strip()
            if not name:
                return
            self.equalizer.save_as_preset(name)
            name_input.value = ""
            select = self.query_one("#eq-preset-select", Select)
            presets = [(p.name, p.name) for p in self.equalizer.get_presets()]
            select.set_options(presets)
        except Exception:
            pass

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
        """Toggle the EQ panel visibility. Auto-focus first band when opening."""
        self.toggle_class("visible")
        if self.has_class("visible"):
            # Auto-focus the first band so user can immediately interact
            try:
                band = self.query_one("#eq-band-0", EQBand)
                band.focus()
            except Exception:
                pass
        else:
            # Return focus to library browser when closing
            try:
                browser = self.screen.query_one("#library-browser")
                browser.focus()
            except Exception:
                pass
