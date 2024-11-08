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

    init() {
      /*addEventListener("focusin", (event) => {
        let parentSlide = event.target.closest("[data-slide-index]");
        if (parentSlide !== null) {
          this.poll.activeSlide = Number(parentSlide.dataset.slideIndex);
          this.save();
        }
        });*/

      addEventListener("keyup", (event) => {
        if (event.target === document.body) {
          if (event.code === "ArrowRight") {
            this.gotoSlide(this.poll.activeSlide + 1);
          } else if (event.code === "ArrowLeft") {
            this.gotoSlide(this.poll.activeSlide - 1);
          }
        }
      });
    },

    save() {
      localStorage.setItem("poll", JSON.stringify(this.poll));
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

    startPoll() {
      fetch("/start_poll", {
        method: "POST",
        body: JSON.stringify(this.poll),
        headers: {
          "Content-type": "application/json; charset=UTF-8",
        },
      })
        .then((response) => response.text())
        .then((code) => console.log(code));

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
