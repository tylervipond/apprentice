let trackIndex = 0;
const tracks = [
  new Audio("resources/dungeon_music_r2.mp3"),
  new Audio("resources/marching_music.mp3"),
];

export function setupAudio() {
  tracks.forEach(
    (track, index) =>
      (track.onended = () => {
        trackIndex = tracks.length % (index + 1);
        tracks[trackIndex].play();
      })
  );
}

export function playAudio() {
  tracks[trackIndex].play();
}

export function pauseAudio() {
  tracks[trackIndex].pause();
}
