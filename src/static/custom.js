let requestsInFlight = 0;

function bindSavingIndicator() {
  requestsInFlight += 1;

  if (requestsInFlight == 1) {
    let btn = document.getElementById("start-poll-btn");
    btn.classList.add("htmx-request");
    btn.disabled = true;
  }
}

function freeSavingIndicator() {
  requestsInFlight -= 1;

  if (requestsInFlight == 0) {
    let btn = document.getElementById("start-poll-btn");
    btn.classList.remove("htmx-request");
    btn.disabled = false;
  }
}

function maybeHideAddAnswerBtn(div_name) {
  let mc_answers_div = document.getElementById(div_name);

  if (mc_answers_div.childElementCount >= 7) {
    let btn = document.querySelector(`#${div_name}>button`);
    btn.remove();
  }
}

function onkeydownMCAnswer(input_element, event, slide_index) {
  /* Enter */
  if (event.keyCode == 13) {
    document.getElementById(`btn-add-answer-${slide_index}`)?.click();
    input_element.blur();
  }
}

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

function submitParticipantNameDialog(event) {
  alert(event);

  event.preventDefault();
  return false;
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
