use std::collections::BTreeSet;

/// Tracks physically held MIDI notes and notes kept alive by the sustain pedal.
#[derive(Debug, Default, Clone)]
pub struct HeldNotes {
    held: BTreeSet<u8>,
    sustained: BTreeSet<u8>,
    pedal_down: bool,
}

impl HeldNotes {
    pub fn handle_key(&mut self, note: u8, pressed: bool) {
        if pressed {
            self.held.insert(note);
            if self.pedal_down {
                self.sustained.insert(note);
            }
        } else {
            self.held.remove(&note);
        }
    }

    /// Updates the sustain pedal state. Returns `true` if the state changed.
    pub fn set_sustain(&mut self, down: bool) -> bool {
        if down == self.pedal_down {
            return false;
        }
        self.pedal_down = down;
        if down {
            self.sustained = self.held.clone();
        } else {
            self.sustained.clear();
        }
        true
    }

    pub fn is_sustained(&self) -> bool {
        self.pedal_down
    }

    /// Physically held notes (excluding released notes held only by sustain).
    pub fn held(&self) -> &BTreeSet<u8> {
        &self.held
    }

    /// Notes feeding the arpeggiator (already sorted in ascending order).
    pub fn active_notes(&self) -> &BTreeSet<u8> {
        if self.pedal_down {
            &self.sustained
        } else {
            &self.held
        }
    }

    /// All notes that should visually light up on the keyboard.
    pub fn lit_notes(&self) -> impl Iterator<Item = u8> + '_ {
        self.sustained.union(&self.held).copied()
    }
}
