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

function onkeydownMCAnswer(input_element, event, item_idx) {
  /* Enter */
  if (event.keyCode == 13) {
    document.getElementById(`btn-add-answer-${item_idx}`)?.click();
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
