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

function createSlide(type) {
  if (type === null) type = "undefined";
  return {
    type: type,
    question: "",
    mcAnswers: [],
    ftAnswers: [],
  };
}

function createPoll() {
  return {
    slides: [
      createSlide("firstSlide"),
      createSlide(null),
      createSlide("lastSlide"),
    ],
    enableLeaderboard: false,
    allowCustomNames: false,
    activeSlide: 1,
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
    isLive: false,
    code: null,
    qrCode: null,

    save() {
      localStorage.setItem("poll", JSON.stringify(this.poll));
    },

    startPoll() {
      this.isLive = true;
      this.code = "1234";
    },

    renderQRCode(el, code) {
      let link;
      if (code === null) link = "http://svoote.com";
      else link = "http://svoote.com/p?c=" + code;

      if (this.qrCode === null) {
        this.qrCode = new QRCode(el, {
          text: link,
          width: 256,
          height: 256,
          colorDark: "#1e293b",
          colorLight: "#ffffff",
          correctLevel: QRCode.CorrectLevel.L,
        });
      } else {
        this.qrCode.clear();
        this.qrCode.makeCode(link);
      }
    },
  }));
});
