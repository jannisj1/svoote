function incrementChar(c, add) {
  return String.fromCharCode(c.charCodeAt(0) + add);
}

function createSlide(type) {
  return {
    type: type,
    question: "",
    mcAnswers: [],
    ftAnswers: [],
    stats: null,
  };
}

function createPoll() {
  return {
    slides: [
      createSlide("firstSlide"),
      createSlide("undefined"),
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
    socket: null,

    init() {
      addEventListener("keyup", (event) => {
        if (event.target === document.body) {
          if (event.code === "ArrowRight") {
            this.gotoSlide(this.poll.activeSlide + 1);
          } else if (event.code === "ArrowLeft") {
            this.gotoSlide(this.poll.activeSlide - 1);
          }
        }
      });

      if (document.pollAlreadyLive === true) {
        this.startPoll();
      }
    },

    save() {
      localStorage.setItem("poll", JSON.stringify(this.poll));
    },

    reset() {
      if (this.isLive == false) {
        this.poll = createPoll();
        this.save();
      }
    },

    gotoSlide(slideIndex) {
      slideIndex = Math.max(
        0,
        Math.min(slideIndex, this.poll.slides.length - 1),
      );
      this.poll.activeSlide = slideIndex;
      this.save();
    },

    questionInputEnterEvent(slideIndex, slide) {
      if (slide.type == "mc") {
        let e = document.getElementById("s-" + slideIndex + "-mc-answer-0");
        if (e !== null) e.focus();
        else document.getElementById("add-mc-answer-" + slideIndex).click();
      } else if (slide.type == "ft") {
        let e = document.getElementById("s-" + slideIndex + "-ft-answer-0");
        if (e !== null) e.focus();
        else document.getElementById("add-ft-answer-" + slideIndex).click();
      }
    },

    async startPoll() {
      let response = await fetch("/start_poll", {
        method: "POST",
        body: JSON.stringify(this.poll),
        headers: {
          "Content-type": "application/json; charset=UTF-8",
        },
      });

      if (response.ok) {
        this.code = await response.text();
        this.isLive = true;
        const wsUrl = `${window.location.protocol === "https:" ? "wss" : "ws"}://${window.location.host}/ws/host/${this.code}`;
        this.socket = new ReconnectingWebSocket(wsUrl);
        this.socket.onopen = (e) => {};
        this.socket.onmessage = (e) => {
          console.log(e.data);
        };
      }
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

    async stopPoll() {
      let response = await fetch("/stop_poll/" + this.code, {
        method: "POST",
      });

      if (response.ok) {
        this.code = null;
        this.isLive = false;
      }
    },
  }));
});
