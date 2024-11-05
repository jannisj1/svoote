let demoAnimationsStarted = false;

function initStartPageDemoAnimations() {
  let observer = new IntersectionObserver(function (entries, observer) {
    if (entries[0].isIntersecting && !demoAnimationsStarted) {
      demoAnimationsStarted = true;
      tickDemoElement(0, 50, "demo-mc-container");
      tickDemoElement(0, 50, "demo-ft-container");
    }
  });

  observer.observe(document.querySelector("#features"));
}

function tickDemoElement(count, limit, elementId) {
  document.getElementById(elementId).dispatchEvent(new Event("demoTick"));

  if (count < limit) {
    setTimeout(
      () => {
        tickDemoElement(count + 1, limit, elementId);
      },
      700 + Math.random() * 3800,
    );
  }
}

function incrementChar(c, add) {
  return String.fromCharCode(c.charCodeAt(0) + add);
}

function createSlide() {
  return {
    type: "undefined",
    question: "",
    mcAnswers: [],
    ftAnswers: [],
  };
}

function createPoll() {
  return {
    slides: [createSlide()],
    enableLeaderboard: false,
    allowCustomNames: false,
    activeSlide: 0,
  };
}

function loadPollFromLocalStorage() {
  let poll = JSON.parse(localStorage.getItem("poll"));

  if (poll !== null) {
    return poll;
  } else return createPoll();
}

document.addEventListener("alpine:init", () => {
  Alpine.data("poll", () => ({
    poll: loadPollFromLocalStorage(),

    save() {
      localStorage.setItem("poll", JSON.stringify(this.poll));
    },
  }));
});
