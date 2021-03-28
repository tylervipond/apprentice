import localforage from 'localforage';

const KEY = "apprentice";
let data = null;

export function save_game_data(d) {
  data = d;
  localforage.setItem(KEY, d);
}

export function load_game_data() {
  return data;
}
export function delete_game_data() {
  data = null;
  localforage.removeItem(KEY);
}

export function has_game_data() {
  return Boolean(data);
}

localforage.getItem(KEY).then((d) => {
  data = d;
});
