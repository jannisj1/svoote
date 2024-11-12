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
    slides: [createSlide("undefined")],
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
    gridView: false,
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

    calculateSlideStyle(slideIndex, activeSlide, gridView) {
      if (!gridView)
        return (
          "transform: perspective(100px)" +
          "translateX(" +
          (slideIndex - activeSlide) * 106 +
          "%)" +
          "translateZ(" +
          (slideIndex == activeSlide ? "0" : "-10") +
          "px)"
        );
      else
        return (
          "transform: perspective(100px)" +
          "translateX(" +
          ((slideIndex % 3) - 1) * 105 +
          "%)" +
          "translateY(" +
          (Math.floor(slideIndex / 3) * 110 - 100) +
          "%)" +
          "translateZ(-210px)"
        );
    },

    gotoSlide(slideIndex) {
      slideIndex = Math.max(
        0,
        Math.min(slideIndex, this.poll.slides.length - 1),
      );
      this.poll.activeSlide = slideIndex;
      this.save();

      if (this.isLive) {
        this.socket.send(
          JSON.stringify({
            cmd: "gotoSlide",
            data: { slideIndex: this.poll.activeSlide },
          }),
        );
      }
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
        this.socket.onopen = (_e) => {
          this.gotoSlide(0);
        };
        this.socket.onmessage = (e) => {
          let msg = JSON.parse(e.data);

          switch (msg.cmd) {
            case "updateStats":
              console.log(msg);
              this.poll.slides[msg.data.slideIndex].stats = msg.data.stats;
              break;
          }
        };
      }
    },

    async stopPoll() {
      let response = await fetch("/stop_poll/" + this.code, {
        method: "POST",
      });

      if (response.ok) {
        this.code = null;
        this.isLive = false;
        this.socket.close();
      }
    },
  }));

  Alpine.data("qrCode", () => ({
    qrCodeObj: null,

    render(el, code) {
      let link = `${window.location.protocol}//${window.location.host}/${code !== null ? "p?c=" + code : ""}`;

      if (this.qrCodeObj === null) {
        this.qrCodeObj = new QRCode(el, {
          text: link,
          width: 256,
          height: 256,
          colorDark: "#334155",
          colorLight: "#ffffff",
          correctLevel: QRCode.CorrectLevel.L,
        });
      } else {
        this.qrCodeObj.clear();
        this.qrCodeObj.makeCode(link);
      }
    },
  }));
});
