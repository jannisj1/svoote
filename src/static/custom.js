function setLang(lang) {
  document.cookie = `lang=${lang}; path=/;`;
  location.reload();
}

async function joinPoll() {
  let e = document.getElementById("poll-id-input");
  let res = await fetch("/poll_exists/" + e.value);
  if (res.ok) {
    let txt = await res.text();
    if (txt == "true") {
      window.location.href = "/p?c=" + e.value;
      return;
    }
  }

  e.classList.add("bg-red-100");
}

function incrementChar(c, add) {
  return String.fromCharCode(c.charCodeAt(0) + add);
}

function homeFromTemplate(variant) {
  let poll = null;
  if (variant == "mc") {
    let slide = createSlide("mc");
    slide.question = "How do you feel about the upcoming exam?";
    slide.mcAnswers = [
      { text: "No problem", isCorrect: false },
      { text: "Didn't learn enough", isCorrect: false },
      { text: "We will see", isCorrect: false },
    ];
    poll = createPoll();
    poll.slides = [slide];
  }

  if (variant == "ft") {
    let slide = createSlide("ft");
    slide.question = "What is your favorite movie character?";
    poll = createPoll();
    poll.slides = [slide];
  }

  localStorage.setItem("poll", JSON.stringify(poll));
  location.href = "/host";
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
    isFullscreen: false,
    code: null,
    socket: null,
    fontSize: "large",

    init() {
      addEventListener("keydown", (event) => {
        if (event.target === document.body) {
          if (event.code === "ArrowRight" || event.code === "Space") {
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
        this.fontSize = "medium";
      }
    },

    calculateSlideClasses(slideIndex, activeSlide, gridView) {
      let classes =
        "absolute inset-0 size-full px-[1.5em] sm:px-[3.5em] pb-[2.5em] pt-[3.5em] flex gap-[3.5em] bg-white border rounded transition-transform duration-500 ease-out transform-gpu ";

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
        "absolute left-1/2 top-[0.25em] -translate-x-1/2 px-[0.875em] py-[0.5em] flex justify-center items-center gap-[0.5em] text-nowrap rounded-full hover:shadow transition duration-300 ";

      if (showSelection) {
        classes += "shadow z-10 bg-slate-700 text-slate-100 ";
        switch (buttonType) {
          case "mc":
            classes += "translate-y-[3em] ";
            break;
          case "ft":
            classes += "translate-y-[6.5em] ";
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

    renderWordCloud(slideIndex) {
      let container = document.getElementById("word-cloud-" + slideIndex);
      if (!container || !this.poll.slides[slideIndex].stats || this.gridView)
        return;

      const stats = this.poll.slides[slideIndex].stats;
      const { width: containerWidth, height: containerHeight } =
        container.getBoundingClientRect();
      const HORIZONTAL_GAP = 36 * this.getFontScale();
      const VERTICAL_GAP = 20 * this.getFontScale();

      const sortedTerms = stats.terms
        .map((term, index) => {
          let c = container.children[index] || document.createElement("div");
          if (!container.children[index]) {
            c.className =
              "absolute size-fit left-1/2 top-full leading-none whitespace-nowrap transition-all duration-500 ease-out invisible";
            c.classList.add(
              [
                "text-rose-600",
                "text-cyan-600",
                "text-lime-600",
                "text-fuchsia-600",
                "text-slate-600",
                "text-teal-600",
              ][index % 6],
            );
            container.appendChild(c);
          }

          c.innerText = term[0];
          c.title = `${term[0]}: ${term[1]}`;
          c.style.fontSize = `${0.5 + (2.25 * term[1]) / stats.maxCount}em`;
          c.style.opacity = `${0.7 + (0.3 * term[1]) / stats.maxCount}`;
          c.style.letterSpacing = `${0.02 - 0.04 * (term[1] / stats.maxCount)}em`;
          c.style.fontWeight = "500";

          return {
            term,
            element: c,
            width: c.offsetWidth,
            height: c.offsetHeight,
          };
        })
        .sort((a, b) => b.term[1] - a.term[1]);

      let rows = [];
      let rowHeightSum = 0;

      // Place terms into rows, prioritizing less filled rows but keeping middle rows denser.
      for (let termIndex = 0; termIndex < sortedTerms.length; termIndex++) {
        const term = sortedTerms[termIndex];
        let placed = false;
        term.element.classList.remove("invisible");

        // Try placing the term into the least filled row
        rows.sort((a, b) => a.width - b.width); // Sort rows by width (ascending)
        for (let row of rows) {
          if (
            row.width + term.width + HORIZONTAL_GAP <= containerWidth &&
            rows.length > 1 &&
            !(rows.length < 3 && termIndex >= 6)
          ) {
            if (row.terms.length % 2 == 1) row.terms.push(term);
            else row.terms.unshift(term);
            row.width += term.width + HORIZONTAL_GAP;
            placed = true;
            break;
          }
        }

        // If no suitable row, create a new one if there's vertical space
        if (!placed) {
          if (rowHeightSum + term.height + VERTICAL_GAP <= containerHeight) {
            const newRow = {
              terms: [term],
              width: term.width,
              height: term.height + VERTICAL_GAP,
            };
            rows.push(newRow);
            rowHeightSum += newRow.height;
          } else {
            term.element.classList.add("invisible"); // Hide terms if no space
          }
        }
      }

      // Sort rows for rendering: middle rows first, alternate outward
      rows.sort((a, b) => b.height - a.height); // Sort rows by width (ascending)
      let rowSequence = [];
      let addBack = true;
      for (i = 0; i < rows.length; i++) {
        if (addBack) rowSequence.push(i);
        else rowSequence.unshift(i);
        addBack = !addBack;
      }

      // Position terms in the container
      let top = (containerHeight - rowHeightSum) / 2;
      for (const rowIndex of rowSequence) {
        const row = rows[rowIndex];
        let leftOffset = containerWidth / 2 - row.width / 2;
        for (const term of row.terms) {
          term.element.style.top = `${top + (row.height - term.height) / 2}px`;
          term.element.style.left = `${leftOffset}px`;
          leftOffset += term.width + HORIZONTAL_GAP;
        }
        top += row.height;
      }

      /*let rows = [];
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
          term.element.style.top = `${top + (row.height - term.height) / 2}px`;
          term.element.style.left = `${leftOffset}px`;
          leftOffset += term.width + HORIZONTAL_GAP;
        }

        top += row.height;
        }*/
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
      if (!before) targetIndex += 1;
      let temp = this.poll.slides[this.reorderedSlideIndex];
      this.poll.slides.splice(targetIndex, 0, temp);

      if (targetIndex < this.reorderedSlideIndex)
        this.poll.slides.splice(this.reorderedSlideIndex + 1, 1);
      else this.poll.slides.splice(this.reorderedSlideIndex, 1);
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
        if (this.gridView) this.gridView = false;

        this.socket = new ReconnectingWebSocket(wsUrl);
        this.socket.onopen = (_e) => {
          this.gotoSlide(this.poll.activeSlide);
        };
        this.socket.onmessage = (e) => {
          let msg = JSON.parse(e.data);

          switch (msg.cmd) {
            case "updateStats":
              this.poll.slides[msg.data.slideIndex].stats = msg.data.stats;
              this.renderWordCloud(msg.data.slideIndex);
              setTimeout(() => this.renderWordCloud(msg.data.slideIndex), 500);
              break;
            case "setEmojiCounts":
              this.poll.slides[msg.data.slideIndex].emojis = msg.data.emojis;
              break;
            case "newEmoji":
              this.poll.slides[msg.data.slideIndex].emojis[msg.data.emoji] += 1;
              setTimeout(() => {
                const emojiMap = {
                  heart: "❤️",
                  thumbsUp: "👍",
                  thumbsDown: "👎",
                  smileyFace: "😀",
                  sadFace: "🙁",
                };

                let el = document.getElementById(
                  "emoji-counter-" + msg.data.emoji,
                );

                const floatingDiv = document.createElement("div");
                floatingDiv.innerText = emojiMap[msg.data.emoji] || "";
                floatingDiv.classList.add(
                  "absolute",
                  "left-[0.5em]",
                  "top-[0.25em]",
                  "text-[1em]",
                  "pointer-events-none",
                  "transition",
                  "duration-500",
                  "opacity-0",
                );

                el.appendChild(floatingDiv);

                requestAnimationFrame(() => {
                  floatingDiv.style.transform = "translateY(-4.5rem)";
                  floatingDiv.style.opacity = "1";
                });

                setTimeout(() => {
                  floatingDiv.style.opacity = "0";
                }, 500);

                setTimeout(() => {
                  floatingDiv.remove();
                }, 1500);
              }, 50);
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

        if (this.isFullscreen) {
          this.toggleFullscreen();
        }
      }
    },

    clearStatistics() {
      for (i = 0; i < this.poll.slides.length; i++) {
        this.poll.slides[i].stats = null;
        this.poll.slides[i].emojis = null;
        let wc = document.getElementById(`word-cloud-${i}`);
        if (wc != null) {
          wc.innerHTML = "";
        }
      }
      this.save();
    },

    toggleFullscreen() {
      if (!document.fullscreenElement) {
        document.getElementById("fullscreen-container").requestFullscreen();
        document.activeElement?.blur(); // Remove focus from fullscreen-button so the user goes to the next slide on pressing space next
      } else if (document.exitFullscreen) document.exitFullscreen();
    },

    getFontScale() {
      if (this.isFullscreen)
        return this.fontSize == "large"
          ? 1.4
          : this.fontSize == "xlarge"
            ? 1.8
            : 1.0;
      else return 1.0;
    },

    async runDemo() {
      function sleep(ms) {
        return new Promise((resolve) => setTimeout(resolve, ms));
      }

      let slide1 = createSlide("mc");
      slide1.question = "How do you feel about the upcoming exam?";
      slide1.mcAnswers = [
        { text: "No problem", isCorrect: false },
        { text: "Didn't learn enough", isCorrect: false },
        { text: "We will see", isCorrect: false },
      ];

      let slide2 = createSlide("ft");
      slide2.question = "What is your favorite movie character?";

      poll = createPoll();
      poll.slides = [slide1, slide2];
      this.poll = poll;
      this.isLive = true;
      this.code = 1234;

      await sleep(1000);
      this.poll.slides[0].stats = {
        percentages: [100, 2, 2],
        counts: [1, 0, 0],
      };
      await sleep(1200);

      let s = this.poll.slides[0].stats;

      const sequence = [
        [2, 1200],
        [2, 1300],
        [0, 1000],
        [0, 1500],
        [1, 1500],
        [0, 700],
        [2, 1200],
        [0, 400],
        [2, 600],
      ];
      for (const el of sequence) {
        s.counts[el[0]] += 1;

        let max = Math.max(...s.counts);
        for (i = 0; i < s.counts.length; i++) {
          s.percentages[i] = s.counts[i] == 0 ? 2 : (s.counts[i] / max) * 100;
        }
        await sleep(el[1]);
      }

      await sleep(1500);
      this.poll.activeSlide = 1;
      await sleep(2500);

      this.poll.slides[1].stats = { terms: [], maxCount: 1 };
      s = this.poll.slides[1].stats;

      const sequenceFt = [
        "Wonder Woman",
        "Indiana Jones",
        "Yoda",
        "Wonder Woman",
        "Neo",
        "Harley Quinn",
        "Wonder Woman",
        "Yoda",
        "Indiana Jones",
        "Shrek",
        "Severus Snape",
        "Batman",
        "Neo",
        "Harley Quinn",
        "Lara Croft",
        "Iron Man",
        "Indiana Jones",
        "Darth Vader",
        "James Bond",
        "Gandalf",
        "Katniss Everdeen",
        "Katniss Everdeen",
      ];

      for (newTerm of sequenceFt) {
        let found = false;
        for (t of s.terms) {
          if (t[0] == newTerm) {
            found = true;
            t[1] += 1;
            if (t[1] > s.maxCount) s.maxCount = t[1];
          }
        }
        if (!found) s.terms.push([newTerm, 1]);

        this.renderWordCloud(1);
        await sleep(600);
        this.renderWordCloud(1);
        await sleep(200 + Math.random() * 600);
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

    async submitEmoji(poll_id, emoji) {
      this.currentSlide.emoji = emoji;
      let res = await fetch("/submit_emoji/" + poll_id, {
        method: "POST",
        headers: { "Content-Type": "application/x-www-form-urlencoded" },
        body: new URLSearchParams({
          emoji: emoji,
          slide_index: this.slideIndex,
        }),
      });

      if (!res.ok) this.currentSlide.emoji = null;
    },
  }));
});
