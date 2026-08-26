// Dinaminis maršrutas (`[id]`) — crawler'is negali atrasti konkrečių ID reikšmių statinio
// build'o metu (SSR išjungtas, duomenys kraunami klientinėje pusėje), tad prerender'inimą
// tenka išjungti šiam maršrutui; runtime'e veikia per adapter-static SPA fallback (`index.html`,
// žr. svelte.config.js).
export const prerender = false;
