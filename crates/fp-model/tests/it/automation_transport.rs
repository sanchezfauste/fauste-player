#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation rework spec §4–§5: the Log and the music fill are driven by
//! the player transport; next is what the operator selected.

use fp_model::automation::ItemRef;
use fp_model::{Command, TrackId};

use crate::automation_rig::{Rig, today};

fn two_blocks() -> Rig {
    use crate::automation_rig::song;
    Rig::new(vec![
        (10, 0, vec![song(1), song(2)]),
        (11, 0, vec![song(3)]),
    ])
}

fn next(r: &Rig) -> Option<ItemRef> {
    r.s.automation.as_ref().unwrap().next
}

fn running(r: &Rig) -> bool {
    r.s.automation.as_ref().unwrap().running
}

/// Started before 10:00, song 1 of the 10:00 block on air.
fn on_air_at_ten() -> Rig {
    let mut r = two_blocks();
    r.at_hms(9, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 0, 0);
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(1)));
    r
}

// Task 1: AR10, AR12.

#[test]
fn stopping_after_an_item_leaves_next_on_the_row_below() {
    let mut r = on_air_at_ten();
    let log = r.schedule_player();
    r.apply(Command::StopAutomationAfterCurrent);
    r.end_current(log);
    r.at_hms(10, 30, 0);
    assert!(!running(&r));
    assert_eq!(next(&r), Some(r.item(today(), 10, 0, 1)));
    assert!(r.s.automation.as_ref().unwrap().next_by_hand);
    r.at_hms(10, 45, 0);
    assert_eq!(next(&r), Some(r.item(today(), 10, 0, 1)));
}

#[test]
fn a_pinned_next_survives_a_clock_jump() {
    let mut r = on_air_at_ten();
    let log = r.schedule_player();
    r.apply(Command::StopAutomationAfterCurrent);
    r.end_current(log);
    r.jump_hms(12, 0, 0);
    assert_eq!(next(&r), Some(r.item(today(), 10, 0, 1)));
}

#[test]
fn at_start_up_next_follows_the_clock() {
    let mut r = two_blocks();
    r.at_hms(10, 30, 0);
    assert_eq!(next(&r), Some(r.item(today(), 11, 0, 0)));
    r.at_hms(10, 50, 0);
    assert_eq!(next(&r), Some(r.item(today(), 11, 0, 0)));
}

// Task 2: AR6, the Log's Stop at end.

#[test]
fn stop_at_end_on_an_item_stops_after_it() {
    let mut r = on_air_at_ten();
    let log = r.schedule_player();
    r.apply(Command::ToggleStopAfterCurrent(log));
    assert!(running(&r), "running until the item ends");
    assert!(r.s.automation.as_ref().unwrap().stop_at_end);
    assert!(r.player(log).stop_after_current);
    r.end_current(log);
    r.at_hms(10, 0, 1);
    assert!(!running(&r));
    assert!(!r.s.automation.as_ref().unwrap().stop_at_end);
    r.at_hms(10, 3, 0);
    assert_eq!(r.playing(log), None);
    assert_eq!(r.playing(r.filler_player()), None);
}

#[test]
fn stop_at_end_during_music_lets_the_song_end_then_stops() {
    let mut r = two_blocks();
    r.at_hms(10, 30, 0);
    r.apply(Command::StartAutomation);
    let f = r.filler_player();
    assert!(r.playing(f).is_some());
    r.apply(Command::ToggleStopAfterCurrent(r.schedule_player()));
    assert!(r.player(f).stop_after_current);
    r.end_current(f);
    r.at_hms(10, 31, 0);
    assert_eq!(r.playing(f), None);
    assert!(!running(&r));
    assert_eq!(next(&r), Some(r.item(today(), 11, 0, 0)));
    r.at_hms(11, 0, 0);
    assert_eq!(r.playing(r.schedule_player()), None);
}

#[test]
fn stop_at_end_during_music_moves_to_the_block_when_it_comes_first() {
    let mut r = two_blocks();
    r.at_hms(10, 58, 30);
    r.apply(Command::StartAutomation);
    let log = r.schedule_player();
    r.apply(Command::ToggleStopAfterCurrent(log));
    r.at_hms(11, 0, 0);
    assert_eq!(r.playing(log), Some(TrackId(3)));
    assert!(r.player(log).stop_after_current);
    assert!(running(&r));
    r.end_current(log);
    r.at_hms(11, 1, 0);
    assert!(!running(&r));
    assert_eq!(r.playing(r.filler_player()), None);
}

#[test]
fn unsetting_stop_at_end_lets_automation_go_on() {
    let mut r = two_blocks();
    r.at_hms(10, 58, 30);
    r.apply(Command::StartAutomation);
    let log = r.schedule_player();
    r.apply(Command::ToggleStopAfterCurrent(log));
    r.apply(Command::ToggleStopAfterCurrent(log));
    assert!(!r.player(r.filler_player()).stop_after_current);
    r.at_hms(11, 0, 0);
    assert_eq!(r.playing(log), Some(TrackId(3)));
    assert!(!r.player(log).stop_after_current);
    assert!(running(&r));
}

// Task 3: AR3–AR5, AR7b: the Log's Play, Pause, Stop and Fade stop.

#[test]
fn play_on_the_log_starts_automation() {
    let mut r = two_blocks();
    r.at_hms(10, 58, 0);
    r.apply(Command::Play(r.schedule_player()));
    assert!(running(&r));
    assert!(r.playing(r.filler_player()).is_some());
    r.at_hms(11, 0, 0);
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(3)));
}

#[test]
fn play_on_the_log_plays_a_pinned_next_at_once() {
    let mut r = two_blocks();
    r.at_hms(11, 30, 0);
    r.apply(Command::SetScheduleNext(r.item(today(), 10, 0, 0)));
    r.apply(Command::Play(r.schedule_player()));
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(1)));
}

#[test]
fn stop_on_the_log_stops_the_item_and_automation() {
    let mut r = on_air_at_ten();
    let log = r.schedule_player();
    r.apply(Command::Stop(log));
    r.at_hms(10, 0, 1);
    assert!(!running(&r));
    assert_eq!(r.playing(log), None);
    assert_eq!(next(&r), Some(r.item(today(), 10, 0, 1)));
    r.at_hms(10, 5, 0);
    assert_eq!(r.playing(log), None);
    assert_eq!(r.playing(r.filler_player()), None);
}

#[test]
fn fade_stop_on_the_log_fades_the_item_and_stops_automation() {
    let mut r = on_air_at_ten();
    let log = r.schedule_player();
    let actions = r.apply(Command::FadeStop(log));
    assert!(actions.iter().any(
        |a| matches!(a, fp_model::EngineAction::FadeOutAndStop { player, .. } if *player == log)
    ));
    r.at_hms(10, 0, 1);
    assert!(!running(&r));
    assert_eq!(r.playing(log), None);
}

#[test]
fn stop_on_the_log_during_music_stops_the_music() {
    let mut r = two_blocks();
    r.at_hms(10, 30, 0);
    r.apply(Command::Play(r.schedule_player()));
    assert!(r.playing(r.filler_player()).is_some());
    r.apply(Command::Stop(r.schedule_player()));
    r.at_hms(10, 30, 1);
    assert!(!running(&r));
    assert_eq!(r.playing(r.filler_player()), None);
    assert_eq!(next(&r), Some(r.item(today(), 11, 0, 0)));
}

#[test]
fn pause_and_play_on_the_log_resume_the_item() {
    let mut r = on_air_at_ten();
    let log = r.schedule_player();
    r.apply(Command::Pause(log));
    assert!(!running(&r));
    assert_eq!(r.player(log).transport, fp_model::Transport::Paused);
    r.apply(Command::Play(log));
    assert!(running(&r));
    assert_eq!(r.playing(log), Some(TrackId(1)));
}

#[test]
fn pause_on_the_log_during_music_pauses_the_filler_and_play_resumes_it() {
    let mut r = two_blocks();
    r.at_hms(10, 30, 0);
    r.apply(Command::Play(r.schedule_player()));
    let f = r.filler_player();
    let song = r.playing(f);
    r.apply(Command::Pause(r.schedule_player()));
    assert!(!running(&r));
    assert_eq!(r.player(f).transport, fp_model::Transport::Paused);
    r.at_hms(10, 30, 5);
    assert_eq!(r.player(f).transport, fp_model::Transport::Paused);
    r.apply(Command::Play(r.schedule_player()));
    assert!(running(&r));
    assert_eq!(r.player(f).transport, fp_model::Transport::Playing);
    assert_eq!(r.playing(f), song);
}

#[test]
fn a_block_start_passing_while_paused_is_missed() {
    let mut r = two_blocks();
    r.at_hms(10, 58, 0);
    r.apply(Command::Play(r.schedule_player()));
    r.apply(Command::Pause(r.schedule_player()));
    r.at_hms(11, 5, 0);
    assert!(r.missed().contains(&r.item(today(), 11, 0, 0)));
    assert_eq!(r.playing(r.schedule_player()), None);
}

#[test]
fn log_stop_leaves_a_filler_playing_on_its_own_alone() {
    let mut r = two_blocks();
    r.at_hms(10, 30, 0);
    let f = r.filler_player();
    r.apply(Command::Play(f));
    assert!(!running(&r));
    r.apply(Command::Stop(r.schedule_player()));
    r.at_hms(10, 30, 1);
    assert!(r.playing(f).is_some());
}

// Task 4: AR7, the music fill's own transport.

/// Automation running at 10:30, the filler's song on air for the Log.
fn music_at_half_past_ten() -> Rig {
    let mut r = two_blocks();
    r.at_hms(10, 30, 0);
    r.apply(Command::Play(r.schedule_player()));
    assert!(r.playing(r.filler_player()).is_some());
    r
}

#[test]
fn stopping_the_filler_during_music_stops_automation() {
    let mut r = music_at_half_past_ten();
    r.apply(Command::Stop(r.filler_player()));
    r.at_hms(10, 30, 1);
    assert!(!running(&r));
    assert_eq!(r.playing(r.filler_player()), None);
    r.at_hms(11, 0, 0);
    assert_eq!(r.playing(r.schedule_player()), None);
}

#[test]
fn the_fillers_stop_at_end_stops_automation_when_its_song_ends_first() {
    let mut r = music_at_half_past_ten();
    let f = r.filler_player();
    r.apply(Command::ToggleStopAfterCurrent(f));
    r.end_current(f);
    r.at_hms(10, 31, 0);
    assert_eq!(r.playing(f), None);
    assert!(!running(&r));
    r.at_hms(11, 0, 0);
    assert_eq!(r.playing(r.schedule_player()), None);
}

#[test]
fn the_fillers_stop_at_end_is_cleared_when_a_block_comes_first() {
    let mut r = two_blocks();
    r.at_hms(10, 58, 30);
    r.apply(Command::Play(r.schedule_player()));
    let f = r.filler_player();
    r.apply(Command::ToggleStopAfterCurrent(f));
    r.at_hms(11, 0, 0);
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(3)));
    assert!(!r.player(f).stop_after_current);
    assert!(running(&r));
}

#[test]
fn play_on_the_filler_while_a_block_plays_mixes() {
    let mut r = on_air_at_ten();
    let f = r.filler_player();
    r.apply(Command::Play(f));
    assert!(r.playing(f).is_some());
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(1)));
    assert!(running(&r));
}

#[test]
fn pausing_the_filler_during_music_stops_automation_and_the_log_resumes_it() {
    let mut r = music_at_half_past_ten();
    let f = r.filler_player();
    r.apply(Command::Pause(f));
    assert!(!running(&r));
    r.apply(Command::Play(r.schedule_player()));
    assert!(running(&r));
    assert_eq!(r.player(f).transport, fp_model::Transport::Playing);
}

// Task 5: AR6b, SINGLE on the Log.

fn single(r: &mut Rig) {
    let log = r.schedule_player();
    r.apply(Command::SetMode(log, fp_model::PlayMode::Single));
}

#[test]
fn single_plays_one_item_and_stops() {
    let mut r = two_blocks();
    single(&mut r);
    r.at_hms(9, 30, 0);
    r.apply(Command::SetScheduleNext(r.item(today(), 10, 0, 0)));
    let log = r.schedule_player();
    r.apply(Command::Play(log));
    assert_eq!(r.playing(log), Some(TrackId(1)));
    r.end_current(log);
    r.at_hms(9, 33, 0);
    assert!(!running(&r));
    assert_eq!(r.playing(log), None);
    assert_eq!(r.playing(r.filler_player()), None);
    assert_eq!(next(&r), Some(r.item(today(), 10, 0, 1)));
}

#[test]
fn single_never_starts_a_block_by_itself() {
    let mut r = two_blocks();
    single(&mut r);
    r.at_hms(10, 58, 0);
    let log = r.schedule_player();
    r.apply(Command::Play(log));
    assert_eq!(r.playing(log), Some(TrackId(3)), "next plays at once");
    r.end_current(log);
    r.at_hms(11, 5, 0);
    assert!(!running(&r));
    assert_eq!(r.playing(r.filler_player()), None);
}

#[test]
fn single_play_on_a_music_fill_row_starts_the_filler_only() {
    use crate::automation_rig::song;
    let mut r = Rig::new(vec![(10, 0, vec![song(1)]), (11, 0, vec![song(3)])]);
    let default = r.s.rotations.default_id().unwrap();
    let base = r.s.schedule.templates()[0].id;
    let mut block =
        r.s.schedule
            .template(base)
            .unwrap()
            .block(r.block(10, 0))
            .unwrap()
            .clone();
    block
        .items
        .push(fp_model::schedule::Item::Rotation { rotation: default });
    r.apply(Command::UpdateBlock {
        template: base,
        block,
    });
    single(&mut r);
    r.at_hms(10, 10, 0);
    r.apply(Command::SetScheduleNext(r.item(today(), 10, 0, 1)));
    r.apply(Command::Play(r.schedule_player()));
    assert!(r.playing(r.filler_player()).is_some());
    assert!(!running(&r));
    r.at_hms(11, 0, 0);
    assert_eq!(r.playing(r.schedule_player()), None);
}

#[test]
fn the_log_mode_survives_a_restart() {
    let mut r = two_blocks();
    single(&mut r);
    let s = &r.s;
    let parts = fp_model::RestoreParts {
        rotations: s.rotations.clone(),
        air_times: Default::default(),
        filler: s.filler_session(|_| 0.0),
        automation: s.automation_session(|_| 0.0),
        schedule: s.schedule.clone(),
        library_view: Default::default(),
        config: s.config.clone(),
        library: s.library.clone(),
        categories: s.categories.clone(),
        playlists: s.playlists.clone(),
        cart_pages: s.cartwall.pages.clone(),
        cartwall_session: s.cartwall.session(),
        layout: s.layout.clone(),
        ids: s.ids.clone(),
    };
    let (back, _) = fp_model::AppState::restore(parts, &s.sessions(|_| 0.0), "Main");
    let log = back.automation.as_ref().unwrap().player;
    let mode = back.players.iter().find(|p| p.id == log).unwrap().mode;
    assert_eq!(mode, fp_model::PlayMode::Single);
    assert!(!back.automation.as_ref().unwrap().running);
}

// Task 6: the Log header's buttons.

#[test]
fn the_log_buttons_follow_automation() {
    let mut r = two_blocks();
    r.at_hms(10, 30, 0);
    let log = r.schedule_player();
    let a = fp_model::availability(&r.s, log);
    assert!(a.play, "a next to play");
    assert!(!a.stop && !a.pause && !a.fade_stop && !a.previous);
    r.apply(Command::Play(log));
    let a = fp_model::availability(&r.s, log);
    assert!(a.play && a.stop && a.pause && a.fade_stop && a.stop_after_current);
    assert!(!a.previous);
    r.apply(Command::Pause(log));
    let a = fp_model::availability(&r.s, log);
    assert!(a.play && a.stop && !a.fade_stop);
}

// Review of rework plan 3.

#[test]
fn a_hand_over_cancelled_during_its_fade_keeps_automation_running() {
    let mut r = two_blocks();
    r.slow_fades = true;
    r.at_hms(10, 58, 30);
    r.apply(Command::Play(r.schedule_player()));
    let f = r.filler_player();
    r.at_hms(10, 59, 59);
    assert!(r.player(f).fade_stopping(), "the hand-over fade runs");
    // A clock correction back: the hand-over no longer counts.
    r.jump_hms(10, 50, 0);
    let current = r.player(f).current.unwrap();
    let more = fp_model::on_event(
        &mut r.s,
        fp_model::EngineEvent::ReachedEnd {
            player: f,
            entry: current,
        },
    );
    r.engine(&more);
    r.at_hms(10, 50, 1);
    assert!(
        running(&r),
        "automation's own fade is not the operator's stop"
    );
    assert!(r.playing(f).is_some(), "the music comes back");
}

#[test]
fn a_stop_during_music_after_back_to_the_clock_keeps_the_clocks_next() {
    let mut r = on_air_at_ten();
    let log = r.schedule_player();
    r.apply(Command::ToggleStopAfterCurrent(log));
    r.end_current(log);
    r.at_hms(10, 55, 0);
    r.apply(Command::ClearScheduleNext);
    r.at_hms(10, 55, 1);
    assert_eq!(next(&r), Some(r.item(today(), 11, 0, 0)));
    r.apply(Command::Play(log));
    assert!(r.playing(r.filler_player()).is_some());
    r.at_hms(10, 57, 0);
    r.apply(Command::Stop(log));
    r.at_hms(10, 57, 1);
    assert_eq!(next(&r), Some(r.item(today(), 11, 0, 0)));
}

#[test]
fn play_clears_a_stop_at_end_it_overrides() {
    let mut r = music_at_half_past_ten();
    let log = r.schedule_player();
    r.apply(Command::ToggleStopAfterCurrent(log));
    r.apply(Command::Play(log));
    assert!(!r.s.automation.as_ref().unwrap().stop_at_end);
    assert!(!r.player(r.filler_player()).stop_after_current);
}

#[test]
fn single_leaves_the_fillers_rotation_alone() {
    use crate::automation_rig::song;
    let mut r = Rig::new(vec![(10, 0, vec![song(1)]), (11, 0, vec![song(3)])]);
    let base = r.s.schedule.templates()[0].id;
    let default = r.s.rotations.default_id().unwrap();
    let mut block =
        r.s.schedule
            .template(base)
            .unwrap()
            .block(r.block(10, 0))
            .unwrap()
            .clone();
    block
        .items
        .push(fp_model::schedule::Item::Rotation { rotation: default });
    r.apply(Command::UpdateBlock {
        template: base,
        block,
    });
    let before = r.s.filler.as_ref().unwrap().rotation;
    single(&mut r);
    r.at_hms(9, 30, 0);
    r.apply(Command::SetScheduleNext(r.item(today(), 10, 0, 0)));
    let log = r.schedule_player();
    r.apply(Command::Play(log));
    r.end_current(log);
    r.at_hms(9, 33, 0);
    assert!(!running(&r));
    assert_eq!(r.s.filler.as_ref().unwrap().rotation, before);
}

#[test]
fn next_stays_where_it_is_while_the_log_is_paused() {
    let mut r = on_air_at_ten();
    let log = r.schedule_player();
    r.apply(Command::Pause(log));
    let before = next(&r);
    assert!(before.is_some());
    r.at_hms(11, 30, 0);
    assert_eq!(next(&r), before);
    assert!(r.s.automation.as_ref().unwrap().next_by_hand);
}

#[test]
fn next_stays_where_it_is_while_the_music_is_paused() {
    let mut r = two_blocks();
    r.at_hms(10, 30, 0);
    r.apply(Command::Play(r.schedule_player()));
    r.apply(Command::Pause(r.filler_player()));
    let before = next(&r);
    assert!(before.is_some());
    r.at_hms(11, 30, 0);
    assert_eq!(next(&r), before);
}

#[test]
fn play_on_the_filler_after_a_log_pause_plays_it_on_its_own() {
    let mut r = two_blocks();
    r.at_hms(10, 30, 0);
    r.apply(Command::Play(r.schedule_player()));
    r.apply(Command::Pause(r.schedule_player()));
    let f = r.filler_player();
    r.apply(Command::Play(f));
    assert!(!running(&r));
    assert!(!r.s.automation.as_ref().unwrap().paused_fill);
    // The Log no longer owns the song: its Stop leaves it playing.
    r.apply(Command::Stop(r.schedule_player()));
    r.at_hms(10, 30, 1);
    assert!(r.playing(f).is_some());
}

#[test]
fn play_during_the_logs_fade_stop_does_nothing_as_on_a_player() {
    let mut r = on_air_at_ten();
    let log = r.schedule_player();
    // The fade is still running: the fake engine does not end it.
    fp_model::apply(&mut r.s, Command::FadeStop(log)).unwrap();
    assert!(r.player(log).fade_stopping());
    let next_before = next(&r);
    r.apply(Command::Play(log));
    assert!(!running(&r));
    assert_eq!(r.playing(log), Some(TrackId(1)));
    assert_eq!(next(&r), next_before);
}
