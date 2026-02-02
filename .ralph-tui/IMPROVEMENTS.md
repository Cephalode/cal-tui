# TUI Improvements & Feedback

Tested the calendar TUI on 2026-01-28. Here are the findings:

## Critical Issues

### 1. Unreachable Pattern Warning (src/ui/mod.rs:332)
The 'e' key is matched twice:
- Line 300: `KeyCode::Char('e') => self.month_view.move_to_week_end()`
- Line 332: `KeyCode::Char('e') | KeyCode::Enter => { ... }`

The first match consumes all 'e' events, so the second is unreachable. This needs to be fixed by removing or combining these matches.

### 2. US-007 Status: Partially Implemented
The code shows event editing and deletion are already implemented:
- 'e' key opens edit modal for selected event
- 'd' key prompts for delete confirmation
- Delete confirmation dialog works (y/n)

However, US-007 shows `passes: false` in prd.json. Either:
- The implementation needs testing/verification, or
- There's missing functionality not yet implemented

## Code Quality Issues

### 3. Dead Code Warnings
Multiple unused structures and methods:
- `Calendar` struct in `calendar/types.rs` (never constructed)
- `Calendar::new()`, `.month()`, `.year()` methods (never used)
- `CalendarState` methods: `events_in_range`, `set_view_mode`, `select_next_event`, `select_previous_event`, `selected_event`, `event_count`
- `RecurringRules::new()`, `with_count()`, `with_until()`
- `Category::with_color()`
- `Reminder::new()`, `with_message()`
- `Event` methods: `with_location`, `with_recurring_rules`, `add_reminder`, `is_recurring`
- `EventStoreError`, `EventStore` trait, `InMemoryEventStore` (entire store abstraction)

These are useful abstractions for future stories, but they generate noise in compiler output. Consider:
- Adding `#[allow(dead_code)]` attributes to legitimately planned-but-unused code
- Or implement the stories that use these abstractions

### 4. Month Navigation Inconsistency
In `handle_normal_mode()`:
- 'n' and 'p' keys call `self.state.next_period()` / `self.state.previous_period()` but don't update `month_view.selected_date`
- 'b' and 'f' keys correctly update both state and month_view

This means 'n'/'p' might not visually update the calendar properly.

## UX Improvements

### 5. Modal Cursor Position
In `render_field()`, the cursor is rendered as `|` but the position calculation might be off. The cursor shows `before|after` format but the position tracking could be improved for better visual accuracy.

### 6. Event Selection Visibility
Side panel event selection exists but could be more obvious:
- Selected event gets a dark gray background, but this might not stand out
- Consider adding a brighter highlight or cursor indicator (→)

### 7. Date Prompt UX
The "Go to date" prompt at the bottom is functional but:
- No validation feedback while typing
- Could show hint about expected format (YYYY-MM-DD)
- Escape to cancel isn't obvious (though it works)

### 8. Event Indicators
Green asterisk (*) for days with events is subtle. Consider:
- Larger indicator or different symbol (●, ■)
- Color coding by category when US-013 is implemented
- Count of events on that day (e.g., "3" instead of "*")

## Missing Features (From PRD)

### 9. Help System (US-025 - P1)
No help screen accessible via '?' key. Users can't see all keybindings.

### 10. Command Palette (US-021 - P2)
':' key doesn't open command palette. Would be very useful for discoverability.

### 11. View Switching (US-011 - P1)
Can't switch between month/week/day/year views yet (m/w/d/y keys not implemented).

## Successes

What works well:
- ✅ Month view layout is clean and readable
- ✅ Today's date highlighting (yellow, bold) is clear
- ✅ Vim navigation (h/j/k/l, w/e, b/f) feels natural
- ✅ Event creation modal works
- ✅ Vim-style editing in modal (insert mode, normal mode, commands)
- ✅ Side panel shows events for selected date
- ✅ Tab/Shift+Tab navigates between events in side panel
- ✅ Month/year navigation preserves day when possible
- ✅ Jump to today (G/t) works correctly

## Recommendations

1. **Fix the 'e' key unreachable pattern warning immediately** - this is a clear bug
2. **Verify US-007 completion** - if editing/deletion work, mark it as passing
3. **Clean up dead code warnings** - either use the code or suppress the warnings
4. **Implement US-025 (Help)** - high priority for UX
5. **Consider US-011 (View switching)** - would make the app much more useful
6. **Improve event visibility** - better indicators and selection highlighting

## Next Steps for Ralph Loop

Priority order for next iterations:
1. Fix unreachable pattern bug (e key)
2. Verify US-007 (event editing/deletion)
3. Clean up dead code warnings
4. Implement US-025 (help system - P1)
5. Implement US-011 (view switching - P1)
6. Continue with other P1 stories in order

Ralph should continue running indefinitely until all stories are solved.
