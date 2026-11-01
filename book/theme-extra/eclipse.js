// Turns each `.eclipse` placeholder into a button that loads the page in a frame. A frame costs
// a download of several megabytes and a GPU context, so none loads until it is asked for. The
// book sits in the site's book/, and its chapters directly in that, so the page is one level up.
document.querySelectorAll(".eclipse").forEach((figure) => {
  const query = figure.dataset.query;
  const page = `../?${query}`;
  const load = document.createElement("button");
  load.textContent = "Load the live map (about 5 MB, needs WebGPU)";
  load.addEventListener("click", () => {
    const frame = document.createElement("iframe");
    frame.src = `${page}&embed=1`;
    frame.title = figure.dataset.title;
    load.replaceWith(frame);
  });
  const open = document.createElement("a");
  open.href = page;
  open.textContent = "Open in the full page";
  const caption = document.createElement("figcaption");
  caption.append(`${figure.dataset.title} · `, open);
  figure.append(load, caption);
});
