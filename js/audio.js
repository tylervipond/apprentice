let trackIndex = 0;
const tracks = [
  new Audio("resources/dungeon_music_r2.mp3"),
  new Audio("resources/marching_music.mp3"),
  new Audio("resources/app_amb1.mp3"),
  new Audio("resources/apprentice4.mp3"),
];

export function setupAudio() {
  tracks.forEach(
    (track, index) =>
      (track.onended = () => {
        trackIndex = index + 1 === tracks.length ? 0 : index + 1;
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
