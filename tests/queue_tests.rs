use cli_music_player::queue::{QueueManager, RepeatMode};
use cli_music_player::subsonic::Song;

fn make_songs(n: usize) -> Vec<Song> {
    (0..n)
        .map(|i| Song {
            id: format!("song{i}"),
            title: format!("Song {i}"),
            artist: "Test".to_string(),
            album: "Test".to_string(),
            duration: 180,
            ..Default::default()
        })
        .collect()
}

// ── RepeatMode Tests ────────────────────────────────────────────

#[test]
fn test_repeat_mode_cycle() {
    let mode = RepeatMode::Off;
    let mode = mode.cycle();
    assert_eq!(mode, RepeatMode::All);
    let mode = mode.cycle();
    assert_eq!(mode, RepeatMode::One);
    let mode = mode.cycle();
    assert_eq!(mode, RepeatMode::Off);
}

#[test]
fn test_repeat_mode_labels() {
    assert_eq!(RepeatMode::Off.label(), "Off");
    assert_eq!(RepeatMode::All.label(), "All");
    assert_eq!(RepeatMode::One.label(), "One");
}

#[test]
fn test_repeat_mode_icons() {
    assert_eq!(RepeatMode::Off.icon(), "↻");
    assert_eq!(RepeatMode::All.icon(), "↻");
    assert_eq!(RepeatMode::One.icon(), "↻1");
}

// ── QueueManager Tests ──────────────────────────────────────────

#[test]
fn test_initial_state() {
    let queue = QueueManager::new();
    assert!(queue.is_empty());
    assert_eq!(queue.length(), 0);
    assert_eq!(queue.current_index(), -1);
    assert!(queue.current_song().is_none());
    assert!(!queue.shuffle_enabled());
    assert_eq!(queue.repeat_mode(), RepeatMode::Off);
}

#[test]
fn test_set_queue() {
    let mut queue = QueueManager::new();
    let songs = make_songs(10);
    queue.set_queue(songs, 2);

    assert!(!queue.is_empty());
    assert_eq!(queue.length(), 10);
    assert_eq!(queue.current_index(), 2);
    assert_eq!(queue.current_song().unwrap().id, "song2");
}

#[test]
fn test_add_song() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 0);
    let new_song = Song {
        id: "new".to_string(),
        title: "New Song".to_string(),
        artist: "Test".to_string(),
        album: "Test".to_string(),
        duration: 200,
        ..Default::default()
    };

    queue.add(new_song);

    assert_eq!(queue.length(), 6);
    assert_eq!(queue.queue().last().unwrap().id, "new");
}

#[test]
fn test_add_songs() {
    let mut queue = QueueManager::new();
    let songs = make_songs(6);
    queue.set_queue(songs[..3].to_vec(), 0);
    queue.add_songs(songs[3..6].to_vec());
    assert_eq!(queue.length(), 6);
}

#[test]
fn test_add_next() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 2);
    let new_song = Song {
        id: "next".to_string(),
        title: "Next Song".to_string(),
        artist: "Test".to_string(),
        album: "Test".to_string(),
        duration: 150,
        ..Default::default()
    };

    queue.add_next(new_song);

    assert_eq!(queue.length(), 6);
    assert_eq!(queue.queue()[3].id, "next");
}

#[test]
fn test_remove_song() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(10), 5);
    queue.remove(3);

    assert_eq!(queue.length(), 9);
    assert_eq!(queue.current_index(), 4); // Adjusted
}

#[test]
fn test_remove_current_song() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(10), 5);
    queue.remove(5);

    assert_eq!(queue.length(), 9);
    assert!(queue.current_index() >= 0);
}

#[test]
fn test_remove_last_song_makes_empty() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(1), 0);
    queue.remove(0);

    assert!(queue.is_empty());
    assert_eq!(queue.current_index(), -1);
}

#[test]
fn test_clear_queue() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(10), 3);
    queue.clear();

    assert!(queue.is_empty());
    assert_eq!(queue.length(), 0);
    assert_eq!(queue.current_index(), -1);
}

#[test]
fn test_move_song() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 0);
    let song_at_1 = queue.queue()[1].id.clone();

    queue.move_item(1, 3);

    assert_eq!(queue.queue()[3].id, song_at_1);
}

#[test]
fn test_move_current_song_updates_index() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 2);
    queue.move_item(2, 4);

    assert_eq!(queue.current_index(), 4);
}

#[test]
fn test_next_without_repeat() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 0);

    let song = queue.next();
    assert!(song.is_some());
    assert_eq!(song.unwrap().id, "song1");
    assert_eq!(queue.current_index(), 1);
}

#[test]
fn test_next_at_end_without_repeat() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 4);
    let song = queue.next();
    assert!(song.is_none());
}

#[test]
fn test_next_with_repeat_all() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 4);
    queue.set_repeat(RepeatMode::All);

    let song = queue.next();
    assert!(song.is_some());
    assert_eq!(song.unwrap().id, "song0");
    assert_eq!(queue.current_index(), 0);
}

#[test]
fn test_next_with_repeat_one() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 2);
    queue.set_repeat(RepeatMode::One);

    let song = queue.next();
    assert!(song.is_some());
    assert_eq!(song.unwrap().id, "song2");
    assert_eq!(queue.current_index(), 2);
}

#[test]
fn test_previous() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 3);

    let song = queue.previous();
    assert!(song.is_some());
    assert_eq!(song.unwrap().id, "song2");
    assert_eq!(queue.current_index(), 2);
}

#[test]
fn test_previous_at_start() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 0);
    let song = queue.previous();
    assert!(song.is_none());
}

#[test]
fn test_previous_with_repeat_all() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 0);
    queue.set_repeat(RepeatMode::All);

    let song = queue.previous();
    assert!(song.is_some());
    assert_eq!(song.unwrap().id, "song4");
    assert_eq!(queue.current_index(), 4);
}

#[test]
fn test_shuffle_toggle() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(10), 0);

    assert!(!queue.shuffle_enabled());
    queue.toggle_shuffle();
    assert!(queue.shuffle_enabled());
    queue.toggle_shuffle();
    assert!(!queue.shuffle_enabled());
}

#[test]
fn test_shuffle_keeps_current_song_first() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(10), 5);
    let current_id = queue.current_song().unwrap().id.clone();

    queue.set_shuffle(true);

    assert_eq!(queue.current_index(), 0);
    assert_eq!(queue.current_song().unwrap().id, current_id);
}

#[test]
fn test_unshuffle_restores_order() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(10), 3);
    let original_id = queue.current_song().unwrap().id.clone();

    queue.set_shuffle(true);
    queue.set_shuffle(false);

    assert_eq!(queue.current_song().unwrap().id, original_id);
    // Queue should be back in original order
    for (i, song) in queue.queue().iter().enumerate() {
        assert_eq!(song.id, format!("song{i}"));
    }
}

#[test]
fn test_has_next() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 3);
    assert!(queue.has_next());

    queue.set_queue(make_songs(5), 4);
    assert!(!queue.has_next());

    // With repeat, always has next
    queue.set_repeat(RepeatMode::All);
    assert!(queue.has_next());
}

#[test]
fn test_has_prev() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 2);
    assert!(queue.has_prev());

    queue.set_queue(make_songs(5), 0);
    assert!(!queue.has_prev());
}

#[test]
fn test_jump_to_valid_index() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(10), 0);

    let song = queue.jump_to(5);
    assert!(song.is_some());
    assert_eq!(song.unwrap().id, "song5");
    assert_eq!(queue.current_index(), 5);
}

#[test]
fn test_jump_to_invalid_index() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(10), 0);

    assert!(queue.jump_to(100).is_none());
}

#[test]
fn test_jump_to_clears_shuffle_history() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(10), 0);
    queue.set_shuffle(true);
    queue.next(); // Adds to history

    queue.jump_to(5);

    assert!(queue.history().is_empty());
}

#[test]
fn test_get_upcoming() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(10), 0);

    let upcoming = queue.get_upcoming(3);
    assert_eq!(upcoming.len(), 3);
    assert_eq!(upcoming[0].id, "song1");
    assert_eq!(upcoming[1].id, "song2");
    assert_eq!(upcoming[2].id, "song3");
}

#[test]
fn test_total_duration() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 0);
    // Each song is 180 seconds, 5 songs = 900
    assert_eq!(queue.total_duration(), 900);
}

// ── Bug Fix Tests ───────────────────────────────────────────────

#[test]
fn test_remove_adjusts_index_correctly() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 3);

    // Remove item before current
    queue.remove(1);
    assert_eq!(queue.current_index(), 2); // Shifted down

    // Remove item after current — no change
    let idx = queue.current_index();
    queue.remove(3);
    assert_eq!(queue.current_index(), idx);
}

#[test]
fn test_negative_indices_rejected() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 2);

    // move with negative-equivalent indices should be no-op
    // (Rust usize makes this impossible at the type level,
    // but jump_to with out-of-bounds should return None)
    assert!(queue.jump_to(usize::MAX).is_none());
}

#[test]
fn test_next_on_empty_queue() {
    let mut queue = QueueManager::new();
    assert!(queue.next().is_none());
}

#[test]
fn test_previous_on_empty_queue() {
    let mut queue = QueueManager::new();
    assert!(queue.previous().is_none());
}

#[test]
fn test_cycle_repeat() {
    let mut queue = QueueManager::new();
    assert_eq!(queue.repeat_mode(), RepeatMode::Off);

    let mode = queue.cycle_repeat();
    assert_eq!(mode, RepeatMode::All);

    let mode = queue.cycle_repeat();
    assert_eq!(mode, RepeatMode::One);

    let mode = queue.cycle_repeat();
    assert_eq!(mode, RepeatMode::Off);
}

// ── Additional Coverage Tests ───────────────────────────────────

#[test]
fn test_has_next_empty_queue() {
    let queue = QueueManager::new();
    assert!(!queue.has_next());
}

#[test]
fn test_has_prev_empty_queue() {
    let queue = QueueManager::new();
    assert!(!queue.has_prev());
}

#[test]
fn test_has_next_with_repeat_one() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(1), 0);
    queue.set_repeat(RepeatMode::One);
    assert!(queue.has_next());
}

#[test]
fn test_add_songs_to_empty_queue() {
    let mut queue = QueueManager::new();
    queue.add_songs(make_songs(3));
    assert_eq!(queue.length(), 3);
}

#[test]
fn test_add_next_at_end() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(3), 2);
    let new_song = Song {
        id: "next".to_string(),
        title: "Next".to_string(),
        artist: "Test".to_string(),
        album: "Test".to_string(),
        duration: 100,
        ..Default::default()
    };
    queue.add_next(new_song);
    assert_eq!(queue.length(), 4);
    assert_eq!(queue.queue()[3].id, "next");
}

#[test]
fn test_get_upcoming_empty_queue() {
    let queue = QueueManager::new();
    assert!(queue.get_upcoming(5).is_empty());
}

#[test]
fn test_get_upcoming_at_last_song() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(3), 2);
    assert!(queue.get_upcoming(3).is_empty());
}

#[test]
fn test_get_upcoming_count_larger_than_remaining() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 3);
    let upcoming = queue.get_upcoming(10);
    assert_eq!(upcoming.len(), 1);
    assert_eq!(upcoming[0].id, "song4");
}

#[test]
fn test_total_duration_empty() {
    let queue = QueueManager::new();
    assert_eq!(queue.total_duration(), 0);
}

#[test]
fn test_total_duration_single_song() {
    let mut queue = QueueManager::new();
    queue.add(Song {
        id: "1".to_string(),
        title: "Solo".to_string(),
        artist: "Test".to_string(),
        album: "Test".to_string(),
        duration: 300,
        ..Default::default()
    });
    assert_eq!(queue.total_duration(), 300);
}

#[test]
fn test_remove_out_of_bounds() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(3), 1);
    queue.remove(100); // Should be no-op
    assert_eq!(queue.length(), 3);
    assert_eq!(queue.current_index(), 1);
}

#[test]
fn test_move_out_of_bounds() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(3), 1);
    queue.move_item(0, 100); // Should be no-op
    assert_eq!(queue.length(), 3);
}

#[test]
fn test_move_item_before_current_updates_index() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 2);
    // Move item from after current to before current
    queue.move_item(4, 1);
    assert_eq!(queue.current_index(), 3);
}

#[test]
fn test_repeat_mode_as_str() {
    assert_eq!(RepeatMode::Off.as_str(), "off");
    assert_eq!(RepeatMode::All.as_str(), "all");
    assert_eq!(RepeatMode::One.as_str(), "one");
}

#[test]
fn test_repeat_mode_from_config_str_roundtrip() {
    for mode in [RepeatMode::Off, RepeatMode::All, RepeatMode::One] {
        assert_eq!(RepeatMode::from_config_str(mode.as_str()), mode);
    }
}

#[test]
fn test_set_shuffle_idempotent() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(10), 0);
    queue.set_shuffle(true);
    assert!(queue.shuffle_enabled());
    queue.set_shuffle(true); // Already enabled, should be no-op
    assert!(queue.shuffle_enabled());
}

#[test]
fn test_shuffle_previous_with_no_history() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 0);
    queue.set_shuffle(true);
    // No history yet, so previous should return None
    assert!(queue.previous().is_none());
}

#[test]
fn test_default_impl() {
    let queue = QueueManager::default();
    assert!(queue.is_empty());
    assert_eq!(queue.current_index(), -1);
}

// ── Bug Regression: Remove with Shuffle ─────────────────────────

#[test]
fn test_remove_with_shuffle_removes_correct_song() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(10), 0);
    queue.set_shuffle(true);

    // After shuffle, the queue order is different from original.
    // Removing index 3 from shuffled queue should remove that specific
    // song from original_queue by ID, not by position.
    let song_to_remove_id = queue.queue()[3].id.clone();
    queue.remove(3);

    // The removed song should not exist anywhere in the queue
    assert!(queue.queue().iter().all(|s| s.id != song_to_remove_id));
    assert_eq!(queue.length(), 9);
}

#[test]
fn test_remove_with_shuffle_preserves_other_songs() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 0);
    queue.set_shuffle(true);

    // Collect all IDs before removal
    let mut ids_before: Vec<String> = queue.queue().iter().map(|s| s.id.clone()).collect();
    let removed_id = ids_before.remove(2); // Remove the song at index 2

    queue.remove(2);

    // All other songs should still be present
    let ids_after: Vec<String> = queue.queue().iter().map(|s| s.id.clone()).collect();
    for id in &ids_before {
        assert!(ids_after.contains(id), "Song {id} was incorrectly removed");
    }
    assert!(!ids_after.contains(&removed_id));
}

#[test]
fn test_remove_current_with_shuffle_adjusts_index() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 0);
    queue.set_shuffle(true);

    // Current is at index 0
    queue.remove(0);

    // Should still have a valid current index
    if !queue.is_empty() {
        assert!(queue.current_index() >= 0);
        assert!((queue.current_index() as usize) < queue.length());
    }
}

// ── Bug Regression: Shuffle History Cap ─────────────────────────

#[test]
fn test_shuffle_history_is_capped() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(10), 0);
    queue.set_shuffle(true);
    queue.set_repeat(RepeatMode::All);

    // Advance many times to grow history
    for _ in 0..300 {
        queue.next();
    }

    // History should be capped at 200
    assert!(queue.history().len() <= 200);
}

#[test]
fn test_shuffle_history_updated_on_remove() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(10), 0);
    queue.set_shuffle(true);
    queue.set_repeat(RepeatMode::All);

    // Build some history
    for _ in 0..5 {
        queue.next();
    }

    let history_len_before = queue.history().len();
    let current = queue.current_index();

    // Remove a song that's not current
    let remove_idx = if current == 0 { 5 } else { 0 };
    queue.remove(remove_idx as usize);

    // History should not reference the removed index
    for &h in queue.history() {
        assert!(h >= 0);
        assert!((h as usize) < queue.length());
    }
    // History length may be same or less (if removed index was in history)
    assert!(queue.history().len() <= history_len_before);
}

// ── Edge Cases: Sequential Operations ───────────────────────────

#[test]
fn test_remove_all_songs_one_by_one() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 2);

    for _ in 0..5 {
        queue.remove(0);
    }

    assert!(queue.is_empty());
    assert_eq!(queue.current_index(), -1);
    assert!(queue.current_song().is_none());
}

#[test]
fn test_next_then_previous_returns_same_song() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 2);

    let original_id = queue.current_song().unwrap().id.clone();
    queue.next();
    queue.previous();
    assert_eq!(queue.current_song().unwrap().id, original_id);
}

#[test]
fn test_set_queue_clears_previous_state() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(10), 5);
    queue.set_shuffle(true);
    queue.next(); // Build history

    // Set a new queue - should reset everything
    queue.set_queue(make_songs(3), 0);
    assert!(queue.history().is_empty());
    assert_eq!(queue.length(), 3);
}

#[test]
fn test_single_song_queue_repeat_one() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(1), 0);
    queue.set_repeat(RepeatMode::One);

    // Should keep returning the same song
    for _ in 0..10 {
        let song = queue.next().unwrap();
        assert_eq!(song.id, "song0");
    }
}

#[test]
fn test_single_song_queue_repeat_all() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(1), 0);
    queue.set_repeat(RepeatMode::All);

    let song = queue.next().unwrap();
    assert_eq!(song.id, "song0");
}

#[test]
fn test_single_song_shuffle_no_repeat() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(1), 0);
    queue.set_shuffle(true);

    // With only 1 song and no repeat, next should return None
    // (no remaining songs to pick from)
    assert!(queue.next().is_none());
}

#[test]
fn test_shuffle_next_never_repeats_immediately() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(20), 0);
    queue.set_shuffle(true);
    queue.set_repeat(RepeatMode::All);

    // Each next should pick a different song from current
    for _ in 0..50 {
        let current_id = queue.current_song().unwrap().id.clone();
        let next_id = queue.next().unwrap().id.to_string();
        if queue.length() > 1 {
            assert_ne!(next_id, current_id, "Shuffle should not repeat immediately");
        }
    }
}

#[test]
fn test_add_next_on_empty_queue() {
    let mut queue = QueueManager::new();
    let song = Song {
        id: "first".to_string(),
        title: "First".to_string(),
        artist: "Test".to_string(),
        album: "Test".to_string(),
        duration: 100,
        ..Default::default()
    };
    // current_index is -1, so insert_pos = 0
    queue.add_next(song);
    assert_eq!(queue.length(), 1);
    assert_eq!(queue.queue()[0].id, "first");
}

#[test]
fn test_move_same_position_is_noop() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 2);
    let original_order: Vec<String> = queue.queue().iter().map(|s| s.id.clone()).collect();

    queue.move_item(2, 2);

    let new_order: Vec<String> = queue.queue().iter().map(|s| s.id.clone()).collect();
    assert_eq!(original_order, new_order);
    assert_eq!(queue.current_index(), 2);
}

#[test]
fn test_has_prev_with_shuffle_and_history() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(10), 0);
    queue.set_shuffle(true);

    // Initially no history
    assert!(!queue.has_prev());

    // After advancing, history should exist
    queue.next();
    assert!(queue.has_prev());
}

#[test]
fn test_shuffle_previous_walks_full_history() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(10), 0);
    queue.set_shuffle(true);
    queue.set_repeat(RepeatMode::All);

    // Record the path forward
    let mut forward_path = vec![queue.current_song().unwrap().id.clone()];
    for _ in 0..5 {
        forward_path.push(queue.next().unwrap().id.to_string());
    }

    // Walk backwards and verify it matches (reversed)
    for i in (0..5).rev() {
        let prev = queue.previous().unwrap();
        assert_eq!(prev.id, forward_path[i]);
    }
}

#[test]
fn test_clear_then_add_works() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 2);
    queue.clear();

    queue.add(Song {
        id: "new".to_string(),
        title: "New".to_string(),
        artist: "Test".to_string(),
        album: "Test".to_string(),
        duration: 100,
        ..Default::default()
    });

    assert_eq!(queue.length(), 1);
    assert_eq!(queue.queue()[0].id, "new");
}

// ── Concurrent-Style Stress Tests ───────────────────────────────

#[test]
fn test_rapid_next_prev_alternation() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(10), 5);

    for _ in 0..100 {
        queue.next();
        queue.previous();
    }

    // Should not panic or corrupt state
    assert!(queue.current_index() >= 0);
    assert!((queue.current_index() as usize) < queue.length());
}

#[test]
fn test_rapid_shuffle_toggle() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(20), 5);

    for _ in 0..50 {
        queue.toggle_shuffle();
        let _ = queue.next();
    }

    // After even toggles, shuffle should be off
    assert!(!queue.shuffle_enabled());
    // All original songs should still be in the queue
    assert_eq!(queue.length(), 20);
}

#[test]
fn test_rapid_add_remove() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 2);

    for i in 0..20 {
        queue.add(Song {
            id: format!("added{i}"),
            title: format!("Added {i}"),
            artist: "Test".to_string(),
            album: "Test".to_string(),
            duration: 100,
            ..Default::default()
        });
        // Remove from the end
        let last_idx = queue.length() - 1;
        queue.remove(last_idx);
    }

    // Should still have original 5 songs
    assert_eq!(queue.length(), 5);
    assert_eq!(queue.current_index(), 2);
}
