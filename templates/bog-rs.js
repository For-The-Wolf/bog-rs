function reveal(elem_ID) {
  if (document.getElementById(elem_ID).style.display === "none") {
    document.getElementById(elem_ID).style.display = "block";
  } else {
    document.getElementById(elem_ID).style.display = "none";
  }
}

function poll_game() {
  var room_id = document.getElementById("room_id").innerHTML;
  var player_id = document.getElementById("player_id").innerHTML;
  fetch("/poll_game/" + room_id + "/" + player_id, {
      method: 'POST'
    })
    .then(response => response.json())
    .then(data => {
      var data = JSON.parse(data);
      update_results(data);
      var time_box = document.getElementById("timery");
      var secs_remaining = (2 * 60) - data.time.secs;
      var mins = Math.floor(secs_remaining / 60);
      var secs = Math.floor(secs_remaining % 60);
      if (secs < 10) {
        time_box.innerHTML = "Time remaining - 0" + mins + ":0" + secs;
      } else {
        time_box.innerHTML = "Time remaining - 0" + mins + ":" + secs;
      }
      if (!data.status) {
        window.location.href = "/lobby/" + room_id + "/" + player_id;
      }
    });
}

function is_first(){
	var params = window.location.href.split("/");
  var room_id = document.getElementById("password").innerHTML;
  fetch("/poll_lobby/Active/" + room_id)
    .then(response => response.json())
    .then(data => {
      var data = JSON.parse(data);
      console.log(data.is_first);
      if (!data.is_first){
        reveal("game_results")
      }
    });
}

function poll_lobby() {
	var params = window.location.href.split("/");
  var room_id = params[params.length-2];//document.getElementById("password").innerHTML;
  var player_id = document.getElementById("player_id").innerHTML;
  var player_list = document.getElementById("player_table");
  if (!document.getElementById("start").disabled) {
    var button_state = "Active";
  } else {
    var button_state = "Inactive";
  }
  fetch("/poll_lobby/" + button_state + "/" + room_id)
    .then(response => response.json())
    .then(data => {
      var data = JSON.parse(data);
      if (data.status) {
        window.location.href = "/multi_player/" + room_id + "/" + player_id
      }
       delete_children(player_list);
      var sorted_players = data.player_list;
      sorted_players.sort((a, b) => {
        return b.score - a.score
      });
      for (idx in sorted_players) {
        var row = document.createElement("tr")
        var player = sorted_players[idx];
        var div = document.createElement("td");
        div.width = "50%"
        var h2 = document.createElement("h2");
        var name = document.createTextNode(player.name);
        h2.appendChild(name);
        div.appendChild(h2);
        row.appendChild(div);
        var div_score = document.createElement("td");
        div.width = "50%"
        var h2_score = document.createElement("h2");
        var score = document.createTextNode(player.score);
        h2_score.appendChild(score);
        div_score.appendChild(h2_score);
        if (player.unique_words.length > 0){
          var h2_words = document.createElement("h2");
          var score_str = "Found: ";
          for (word_idx in player.unique_words){
            score_str = score_str + player.unique_words[word_idx] + " ";
          }
          var words = document.createTextNode(score_str);
          h2_words.appendChild(words)
          div_score.appendChild(h2_words);
        }
        row.appendChild(div_score);
        player_list.appendChild(row);
      }
    })
}

function create_lobby() {
  var user_name = document.getElementById("create_room_player_name").value;
  var room_name = document.getElementById("create_room_name").value;
  if (room_name && user_name) {
    fetch("./create_room/" + room_name, {
        method: 'POST'
      })
      .then(response => response.json())
      .then(room_id => {
        if (!room_id.hasOwnProperty("error")) {
          fetch("./insert_player/" + room_id + "/" + user_name, {
              method: 'POST'
            })
            .then(response => response.json())
            .then(player_id => {
              window.location.href = "./lobby/" + room_id + "/" + player_id
            })
        }
      })
  }
}

function join_lobby() {
  var user_name = document.getElementById("join_room_player_name").value;
  var room_id = document.getElementById("join_room_id").value;
  if (user_name && room_id) {
    fetch("./insert_player/" + room_id + "/" + user_name, {
        method: 'POST'
      })
      .then(response => response.json())
      .then(player_id => {
        window.location.href = "./lobby/" + room_id + "/" + player_id
      })
  }
}

function check_guess() {
  var textbox = document.getElementById("eval_answer");
  var room_id = document.getElementById("room_id").innerHTML;
  var player_id = document.getElementById("player_id").innerHTML;
  var word = textbox.value;
  if (word) {
    fetch("/eval_guess/" + room_id + "/" + player_id + "/" + word, {
        method: 'POST'
      })
      .then(response => response.json())
      .then(data => {
        var data = JSON.parse(data);
        update_results(data);
      });
    textbox.value = '';
  }
}

function update_results(data) {
  var results = document.getElementById("guesses");
  delete_children(results);
  var result_table = document.createElement("table");
  result_table.className = "center";
  var count = 0;
  for (idx in data.valid_guesses) {
    if (count % 5 == 0) {
      var row = document.createElement("tr");
    }
    var column = document.createElement("td");
    column.style.padding = "20px";
    var tag = document.createElement("h2");
    var word = data.valid_guesses[idx][0];
    var del = document.createElement("del");
    var text = document.createTextNode(word);
    if (!data.valid_guesses[idx][1]) {
      var text = document.createTextNode(" " + word + " ");
      del.appendChild(text);
      tag.appendChild(del);
    } else {

      tag.appendChild(text);
    }
    column.appendChild(tag);
    row.appendChild(column);
    count++;
    if (count % 5 == 0) {
      result_table.appendChild(row);
    }
    result_table.appendChild(row);
    results.appendChild(result_table);
    document.getElementById("score_box").innerHTML = "Score - " + data.score;
  }
}

function delete_children(node) {
  while (node.firstChild) {
    node.removeChild(node.lastChild)
  }
}

function start_timer() {
  var timer_len = new Date().getTime() + (2 * 60 * 1000);
  var x = setInterval(() => {
    var now = new Date().getTime();
    var distance = timer_len - now;
    var minutes = Math.floor(distance / (1000 * 60));
    var seconds = Math.floor((distance % (1000 * 60)) / 1000);
    if (seconds < 10) {
      document.getElementById("timery").innerHTML = "Time remaining - 0" + minutes + ":0" + seconds;
    } else {
      document.getElementById("timery").innerHTML = "Time remaining - 0" + minutes + ":" + seconds;
    }
    if (distance < 0) {
      clearInterval(x);
      document.getElementById("timery").innerHTML = "TIME'S UP NERDS";
      reveal("solution_zone");
      reveal("submit_form");
    }
  })
}

function and_then_deactivate(butt_ID) {
  document.getElementById(butt_ID).disabled = 'disabled';
  toggle_pressed(butt_ID);
}

function toggle_pressed(butt_ID) {
  var button_source = document.getElementById(butt_ID).src;
  var appendix = "_pressed.png";
  if (button_source.slice(button_source.length - appendix.length) == appendix) {
		document.getElementById(butt_ID).src = "/images/buttons/" + butt_ID + ".png";
  }
	else{
		document.getElementById(butt_ID).src = "/images/buttons/" + butt_ID + appendix;
	}
}

function focus_on(elem_ID) {
  document.getElementById(elem_ID).focus();
}
