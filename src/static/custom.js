function incrementChar(c, add) {
  return String.fromCharCode(c.charCodeAt(0) + add);
}

function createSlide(type) {
  return {
    type: type,
    question: "",
    mcAnswers: [
      { text: "", isCorrect: false },
      { text: "", isCorrect: false },
    ],
    allowMultipleMCAnswers: false,
    ftAnswers: [],
    stats: null,
  };
}

function createPoll() {
  return {
    slides: [createSlide("mc")],
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
    isReordering: false,
    reorderedSlideIndex: null,
    isLive: false,
    code: null,
    socket: null,

    init() {
      addEventListener("keydown", (event) => {
        if (event.target === document.body) {
          if (event.code === "ArrowRight") {
            this.gotoSlide(this.poll.activeSlide + 1);
          } else if (event.code === "ArrowLeft") {
            this.gotoSlide(this.poll.activeSlide - 1);
          }
        }
      });

      this.poll.slides.forEach((slide) => {
        slide.stats = null;
      });

      if (document.pollAlreadyLive === true) {
        this.startPoll();
      }
    },

    save() {
      localStorage.setItem("poll", JSON.stringify(this.poll));
    },

    importJsonFile(inputEvent) {
      const file = inputEvent.target.files[0];

      if (file) {
        const reader = new FileReader();
        reader.onload = (e) => {
          this.poll = JSON.parse(e.target.result);
          this.save();
        };
        reader.onerror = (e) => {
          console.error("Error while reading a poll json file: ", e);
        };
        reader.readAsText(file);
      }
    },

    reset() {
      if (this.isLive == false) {
        this.poll = createPoll();
        this.save();
        this.gridView = false;
        this.isReordering = false;
        this.reorderedSlideIndex = null;
      }
    },

    calculateSlideClasses(slideIndex, activeSlide, gridView) {
      let classes =
        "absolute inset-0 size-full px-6 sm:px-14 pb-10 pt-14 flex gap-14 bg-white border rounded transition-transform duration-500 ease-out transform-gpu ";

      if (gridView) {
        classes +=
          "cursor-pointer shadow-2xl hover:ring-indigo-500 hover:ring-4 ";

        if (slideIndex == activeSlide) classes += "ring-4 ring-indigo-500 ";
        else classes += "ring-2 ring-slate-300 ";
      } else {
        classes += "shadow-lg ";

        if (slideIndex != activeSlide) classes += "cursor-pointer ";
      }

      return classes;
    },

    calculateSlideStyle(slideIndex, activeSlide, gridView, isLive) {
      if (!gridView)
        return (
          "transform: perspective(100px)" +
          "translateX(" +
          (slideIndex - activeSlide) * (isLive ? 120 : 106) +
          "%)" +
          "translateZ(" +
          (slideIndex == activeSlide ? "0" : "-10") +
          "px)"
        );
      else
        return (
          "transform: perspective(100px)" +
          "translateX(" +
          ((slideIndex % 3) - 1) * 120 +
          "%)" +
          "translateY(" +
          (Math.floor(slideIndex / 3) * 150 - 100) +
          "%)" +
          "translateZ(-240px)"
        );
    },

    calculateSlideTypeButtonClasses(slideType, buttonType, showSelection) {
      let classes =
        "absolute left-1/2 top-1 -translate-x-1/2 px-3.5 py-2 flex justify-center items-center gap-2 rounded-full hover:shadow transition duration-300 ";

      if (showSelection) {
        classes += "shadow z-10 bg-slate-700 text-slate-100 ";
        switch (buttonType) {
          case "mc":
            classes += "translate-y-[3rem] ";
            break;
          case "ft":
            classes += "translate-y-[6.5rem] ";
            break;
        }

        if (slideType == buttonType) {
          classes += "ring-4 ring-indigo-500 ";
        }
      } else if (slideType == buttonType) {
        classes += "z-10 scale-75 bg-white text-slate-500 ";
      } else {
        classes += "opacity-0 pointer-events-none scale-75 bg-white ";
      }

      return classes;
    },

    calculateWCTermStyle(term, maxCount) {
      return `
        transition: all 0.5s ease-out;
        font-size: ${0.5 + (2.25 * term.count) / maxCount}rem;
        opacity: ${0.7 + (0.3 * term.count) / maxCount};
        letter-spacing: ${0.02 - 0.04 * (term.count / maxCount)}em;
        font-weight: 500;
        top: ${term.top != null ? term.top : 0}px;
        left: ${term.left != null ? term.left : 0}px;
      `;
    },

    renderWordCloud(slideIndex) {
      let container = document.getElementById("word-cloud-" + slideIndex);
      if (
        container == null ||
        this.poll.slides[slideIndex].stats == null ||
        this.gridView
      )
        return;

      let stats = this.poll.slides[slideIndex].stats;

      let containerHeight = container.getBoundingClientRect().height;
      let containerWidth = container.getBoundingClientRect().width;
      const HORIZONTAL_GAP = 24;
      const VERTICAL_GAP = 12;

      let sortedTerms = [];
      for (i = 0; i < stats.terms.length; i++) {
        let element = container.children[i + 1];
        sortedTerms.push({
          term: stats.terms[i],
          element: element,
          width: element.getBoundingClientRect().width,
          height: element.getBoundingClientRect().height,
        });
      }
      sortedTerms.sort((a, b) => b.term.count - a.term.count);

      let rows = [];
      let rowHeightSum = 0;

      for (termIndex = 0; termIndex < sortedTerms.length; termIndex++) {
        let term = sortedTerms[termIndex];
        let termFoundPlace = false;
        term.element.classList.remove("invisible");

        for (rowIndex = 0; rowIndex < rows.length; rowIndex++) {
          let row = rows[rowIndex];
          if (row.width + term.width + HORIZONTAL_GAP < containerWidth) {
            if (row.terms.length % 2 == 1) row.terms.push(term);
            else row.terms.unshift(term);
            row.width += term.width + HORIZONTAL_GAP;
            termFoundPlace = true;
            break;
          }
        }

        if (!termFoundPlace) {
          if (rowHeightSum + term.height + VERTICAL_GAP <= containerHeight) {
            let height = term.height + VERTICAL_GAP;
            rows.push({
              terms: [term],
              height: height,
              width: term.width,
            });

            rowHeightSum += height;
          } else {
            term.element.classList.add("invisible");
          }
        }
      }

      let rowSequence = [];
      let addBack = true;
      for (i = 0; i < rows.length; i++) {
        if (addBack) rowSequence.push(i);
        else rowSequence.unshift(i);
        addBack = !addBack;
      }

      let top = (containerHeight - rowHeightSum) / 2;

      for (i = 0; i < rows.length; i++) {
        let row = rows[rowSequence[i]];

        let leftOffset = containerWidth / 2 - row.width / 2;
        for (term of row.terms) {
          term.term.top = top + (row.height - term.height) / 2;
          term.term.left = leftOffset;
          leftOffset += term.width + HORIZONTAL_GAP;
        }

        top += row.height;
      }
    },

    gotoSlide(slideIndex) {
      slideIndex = Math.max(
        0,
        Math.min(slideIndex, this.poll.slides.length - 1),
      );
      this.poll.activeSlide = slideIndex;
      this.save();

      window.dispatchEvent(new Event("slidechange"));

      if (this.isLive) {
        this.socket.send(
          JSON.stringify({
            cmd: "gotoSlide",
            data: { slideIndex: this.poll.activeSlide },
          }),
        );
      }
    },

    moveSlide(targetIndex, before) {
      let temp = this.poll.slides.splice(this.reorderedSlideIndex, 1);
      if (targetIndex >= this.reorderedSlideIndex) targetIndex -= 1;
      if (before) this.poll.slides.splice(targetIndex, 0, temp[0]);
      else this.poll.slides.splice(targetIndex + 1, 0, temp[0]);
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
        document.querySelector("body").dataset.live = true;
        const wsUrl = `${window.location.protocol === "https:" ? "wss" : "ws"}://${window.location.host}/ws/host/${this.code}`;

        this.socket = new ReconnectingWebSocket(wsUrl);
        this.socket.onopen = (_e) => {
          this.gotoSlide(this.poll.activeSlide);
        };
        this.socket.onmessage = (e) => {
          let msg = JSON.parse(e.data);

          switch (msg.cmd) {
            case "updateStats":
              this.poll.slides[msg.data.slideIndex].stats = msg.data.stats;
              this.$nextTick(() => this.renderWordCloud(msg.data.slideIndex));
              setTimeout(() => this.renderWordCloud(msg.data.slideIndex), 500);
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
        this.clearStatistics();
        document.querySelector("body").dataset.live = false;
      }
    },

    clearStatistics() {
      this.poll.slides.forEach((slide) => {
        slide.stats = null;
      });
      this.save();
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

  Alpine.data("participant", () => ({
    currentSlide: {
      slideType: "empty",
    },
    slideIndex: null,
    socket: null,

    init() {
      const wsUrl = `${window.location.protocol === "https:" ? "wss" : "ws"}://${window.location.host}/ws/p/${document.code}`;

      this.socket = new ReconnectingWebSocket(wsUrl);
      this.socket.onopen = (_e) => {};
      this.socket.onmessage = (e) => {
        let msg = JSON.parse(e.data);

        switch (msg.cmd) {
          case "updateSlide":
            console.log(msg);
            this.currentSlide = msg.data.slide;
            this.slideIndex = msg.data.slideIndex;
            break;
        }
      };
    },

    async submitMCAnswer(poll_id) {
      let res = await fetch("/submit_mc_answer/" + poll_id, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          answer_indices: this.currentSlide.allowMultipleMCAnswers
            ? this.currentSlide.selectedAnswer.map(Number)
            : [Number(this.currentSlide.selectedAnswer)],
          slide_index: this.slideIndex,
        }),
      });

      if (res.ok) this.currentSlide.submitted = true;
    },

    async submitFTAnswer(poll_id) {
      let res = await fetch("/submit_ft_answer/" + poll_id, {
        method: "POST",
        headers: { "Content-Type": "application/x-www-form-urlencoded" },
        body: new URLSearchParams({
          answer: this.currentSlide.selectedAnswer,
          slide_index: this.slideIndex,
        }),
      });

      if (res.ok) this.currentSlide.submitted = true;
    },
  }));
});
