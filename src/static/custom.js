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
    mcChartType: "bar",
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
  for (const slide of poll.slides) {
    if (slide.mcChartType === undefined) {
      slide.mcChartType = "bar";
    }
  }

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
        "absolute inset-0 size-full px-[1.5em] sm:px-[3.5em] pb-[2.5em] pt-[3.5em] flex gap-[3.5em] bg-white border rounded-xs transition-transform duration-500 ease-out transform-gpu ";

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
    },

    renderPieChart(slideIndex) {
      console.log(slideIndex);
      const canvas = document.getElementById("pie-chart-canvas-" + slideIndex);
      if (!canvas || !canvas.getContext) return;
      console.log(canvas);
      const slide = this.poll.slides[slideIndex];
      console.log(slide);
      const stats = slide.stats;
      console.log(stats);

      const oldPercentages = slide.oldPercentages;
      let newPercentages, newAbsolutes;
      if (stats !== null) {
        newPercentages = stats.percentages;
        newAbsolutes = stats.counts;
      } else {
        const mcAnswers = slide.mcAnswers;
        const length = mcAnswers.length;
        newPercentages = Array(length).fill(100 / length);
        newAbsolutes = Array(length).fill(0);
      }

      const ctx = canvas.getContext("2d");
      const rect = canvas.getBoundingClientRect();
      const width = rect.width;
      const height = rect.height;
      const dpi = window.devicePixelRatio || 1;
      canvas.width = width * dpi;
      canvas.height = height * dpi;
      ctx.scale(dpi, dpi);
      const em = parseFloat(getComputedStyle(canvas).fontSize);
      const radius = Math.min(width, height) / 2 - 2 * em;
      const centerX = width / 2;
      const centerY = height / 2;

      function drawPieChart(percentages, absolutes) {
        ctx.clearRect(0, 0, width, height);
        let startAngle = -0.5 * Math.PI; // Start at the top
        percentages.forEach((percent, index) => {
          if (percent > 0) {
            const sliceAngle = (percent / 100) * 2 * Math.PI;
            ctx.beginPath();
            ctx.moveTo(centerX, centerY);
            console.log(canvas);
            ctx.arc(
              centerX,
              centerY,
              radius,
              startAngle,
              startAngle + sliceAngle,
            );
            ctx.closePath();
            ctx.fillStyle = colorPaletteRGB[index % colorPaletteRGB.length];
            ctx.fill();

            // Draw labels
            const midAngle = startAngle + sliceAngle / 2;
            const countLabelX = centerX + 0.7 * radius * Math.cos(midAngle);
            const countLabelY = centerY + 0.7 * radius * Math.sin(midAngle);
            const answerLabelX = centerX + 1.12 * radius * Math.cos(midAngle);
            const answerLabelY = centerY + 1.12 * radius * Math.sin(midAngle);
            ctx.fillStyle = "#EEE";
            ctx.font = "1.125em Arial";
            ctx.textAlign = "center";
            ctx.textBaseline = "middle";
            ctx.fillText(absolutes[index], countLabelX, countLabelY);

            ctx.font = "0.875em Arial";
            ctx.fillStyle = "#64748b";
            if (Math.cos(midAngle) > 0) ctx.textAlign = "left";
            else ctx.textAlign = "right";
            if (Math.abs(midAngle - Math.PI / 2) <= 0.15)
              ctx.textAlign = "center";

            if (Math.abs(midAngle - 1.5 * Math.PI) <= 0.15) {
              ctx.textBaseline = "bottom";
            } else if (Math.abs(midAngle - 0.5 * Math.PI) <= 0.15) {
              ctx.textBaseline = "top";
            } else if (
              Math.abs(midAngle) <= 0.15 ||
              Math.abs(midAngle - Math.PI) <= 0.15 ||
              Math.abs(midAngle - 2 * Math.PI) <= 0.15
            ) {
              ctx.textBaseline = "middle";
            }

            ctx.fillText(
              slide.mcAnswers[index].text,
              answerLabelX,
              answerLabelY,
            );

            startAngle += sliceAngle;
          }
        });

        // Draw seperation lines if more than one pie slice
        if (
          percentages.length > 1 &&
          !(newAbsolutes.filter((x) => x > 0).length === 1)
        ) {
          startAngle = -0.5 * Math.PI;
          percentages.forEach((percent, index) => {
            if (percent > 0) {
              const sliceAngle = (percent / 100) * 2 * Math.PI;
              ctx.save();
              ctx.translate(centerX, centerY);
              ctx.rotate(startAngle + sliceAngle);
              ctx.fillStyle = "#FFF";
              ctx.fillRect(0, -2, radius, 4);
              ctx.restore();

              startAngle += sliceAngle;
            }
          });

          ctx.beginPath();
          ctx.arc(centerX, centerY, 2, 0, 2 * Math.PI);
          ctx.fillStyle = "#FFF";
          ctx.fill();
        }
      }

      if (oldPercentages) {
        const duration = 500;
        const startTime = performance.now();
        const initialPercentages = oldPercentages.slice();
        while (initialPercentages.length < newPercentages.length)
          initialPercentages.push(0);
        if (initialPercentages.length > newPercentages.length)
          initialPercentages.length = newPercentages.length;
        const deltaPercentages = newPercentages.map(
          (newVal, i) => newVal - initialPercentages[i],
        );

        function animate(currentTime) {
          const elapsed = currentTime - startTime;
          const progress = Math.min(elapsed / duration, 1);
          // Bezier easing function for easeInOut
          const ease =
            progress < 0.5
              ? 4 * progress * progress * progress
              : 1 - Math.pow(-2 * progress + 2, 3) / 2;
          const currentPercentages = initialPercentages.map(
            (start, i) => start + deltaPercentages[i] * ease,
          );
          drawPieChart(currentPercentages, newAbsolutes);
          if (progress < 1) {
            requestAnimationFrame(animate);
          }
        }

        requestAnimationFrame(animate);
      } else {
        drawPieChart(newPercentages, newAbsolutes);
      }

      slide.oldPercentages = newPercentages;
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

        let startBtn = document.getElementById("start-stop-button");
        startBtn.style.width = `${startBtn.offsetWidth}px`;
        this.startBtnWidth = startBtn.style.width;
        startBtn.firstElementChild.style.display = "none";
        requestAnimationFrame(() => {
          startBtn.style.width = "2.25rem";
          startBtn.style.paddingInline = "0.5rem";
        });

        this.socket = new ReconnectingWebSocket(wsUrl);
        this.socket.onopen = (_e) => {
          this.gotoSlide(this.poll.activeSlide);
        };
        this.socket.onmessage = (e) => {
          let msg = JSON.parse(e.data);

          switch (msg.cmd) {
            case "updateStats":
              let slide = this.poll.slides[msg.data.slideIndex];
              const oldStats = slide.stats;
              slide.stats = msg.data.stats;
              if (slide.type == "mc") {
                if (slide.mcChartType == "bar") {
                  slide.stats.percentages = slide.stats.counts
                    .map((count) =>
                      Math.max(...slide.stats.counts) > 0
                        ? (100.0 * count) / Math.max(...slide.stats.counts)
                        : 0,
                    )
                    .map((percent) => (percent === 0 ? 2 : percent));

                  const hasMaxPercentageIncrease =
                    oldStats === null
                      ? false
                      : oldStats.percentages.some(
                          (percentage, i) =>
                            percentage === 100 &&
                            slide.stats.counts[i] > oldStats.counts[i],
                        );
                  if (hasMaxPercentageIncrease) {
                    slide.stats.percentages = slide.stats.percentages.map(
                      (percent) => percent * 1.2,
                    );
                    slide.stats.scaled = true;
                    setTimeout(() => {
                      if (slide.stats.scaled) {
                        slide.stats.percentages = slide.stats.percentages.map(
                          (percent) => percent / 1.2,
                        );
                        delete slide.stats.scaled;
                      }
                    }, 1000);
                  }
                } else if (slide.mcChartType == "pie") {
                  const sum = slide.stats.counts.reduce((a, b) => a + b, 0);
                  slide.stats.percentages = slide.stats.counts.map(
                    (count) => (count / (sum || 1)) * 100,
                  );
                  this.renderPieChart(msg.data.slideIndex);
                }
              } else if (slide.type == "ft") {
                this.renderWordCloud(msg.data.slideIndex);
                setTimeout(
                  () => this.renderWordCloud(msg.data.slideIndex),
                  500,
                );
              }
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

        let startBtn = document.getElementById("start-stop-button");
        startBtn.style.width = `${startBtn.offsetWidth}px`;
        requestAnimationFrame(() => {
          startBtn.style.width = this.startBtnWidth;
          startBtn.style.paddingInline = "";
        });
        setTimeout(() => {
          startBtn.style.width = "";
          startBtn.firstElementChild.style.display = "inline-block";
        }, 200);
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
