// Click on a screenshot to see it at full size; click, Esc or the button
// closes it.  Without JavaScript, the links simply open the image.
(function () {
  "use strict";
  function init() {
    const box = document.createElement("div");
    box.className = "lightbox";
    box.hidden = true;
    box.setAttribute("role", "dialog");
    box.setAttribute("aria-modal", "true");
    box.innerHTML = '<button type="button" class="lightbox-close" aria-label="Close">×</button>' +
                    '<img alt=""><p class="lightbox-caption"></p>';
    document.body.appendChild(box);
    const img = box.querySelector("img");
    const caption = box.querySelector(".lightbox-caption");
    let opener = null;

    function close() {
      box.hidden = true;
      document.body.classList.remove("lightbox-open");
      if (opener) opener.focus();
    }
    box.addEventListener("click", close);
    document.addEventListener("keydown", (e) => { if (e.key === "Escape" && !box.hidden) close(); });

    for (const link of document.querySelectorAll("a.zoom")) {
      link.addEventListener("click", (e) => {
        if (e.ctrlKey || e.metaKey || e.shiftKey || e.button !== 0) return;
        e.preventDefault();
        opener = link;
        const thumb = link.querySelector("img");
        img.src = link.href;
        img.alt = thumb ? thumb.alt : "";
        const fig = link.closest("figure");
        const cap = fig && fig.querySelector("figcaption");
        caption.textContent = cap ? cap.textContent : (thumb ? thumb.alt : "");
        box.hidden = false;
        document.body.classList.add("lightbox-open");
        box.querySelector(".lightbox-close").focus();
      });
    }
  }
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init);
  else init();
})();
