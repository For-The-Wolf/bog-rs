// test
function reveal(elem_ID)
{
		if (document.getElementById(elem_ID).style.display === "none"){
				document.getElementById(elem_ID).style.display="block";
		}
		else{
				document.getElementById(elem_ID).style.display="none";
		}
}
function poll_game(){
	var room_id = document.getElementById("room_id").innerHTML;
	var player_id = document.getElementById("player_id").innerHTML;
	fetch("/beta/poll_game/"+room_id+"/"+player_id,{method:'POST'})
		.then(response => response.json())
		.then(data => {
			var data = JSON.parse(data);
			console.log(data);
			update_results(data);
			var time_box = document.getElementById("timery");
			var secs_remaining = (2*60) - data.time.secs;
			var mins = Math.floor(secs_remaining/60);
			var secs = Math.floor(secs_remaining % 60);
			if (secs < 10){
				time_box.innerHTML = "Time remaining - 0" + mins + ":0" + secs;
			}else{
				time_box.innerHTML = "Time remaining - 0" + mins + ":" + secs;
			}
			if (!data.status) {
			 window.location.href = "/beta/lobby/"+room_id+"/"+player_id
			}
		})
}
function poll_lobby(){
	var room_id = document.getElementById("password").innerHTML;
	var player_id = document.getElementById("player_id").innerHTML;
	var player_list = document.getElementById("player_list");
	var score_list = document.getElementById("score_list");
	if (!document.getElementById("start").disabled){
		var button_state = "Active"
	} else {
		var button_state = "Inactive"
	}
	console.log(button_state);
	fetch("/beta/poll_lobby/"+button_state+"/"+room_id)
		.then(response => response.json())
		.then(data => {
			var data = JSON.parse(data);
			if (data.status){
				window.location.href = "/beta/multi_player/"+room_id+"/"+player_id
			}
			console.log(data);
			delete_children(player_list);
			delete_children(score_list);
			var sorted_players = data.player_list;
			sorted_players.sort((a,b) => {return b[1] - a[1]});
			for (idx in sorted_players){
				var div = document.createElement("div");
				var h2 = document.createElement("h2");
				var name = document.createTextNode(sorted_players[idx][0]);
				h2.appendChild(name);
				div.appendChild(h2);
				player_list.appendChild(div);
				var div_score = document.createElement("div");
				var h2_score = document.createElement("h2");
				var score = document.createTextNode(sorted_players[idx][1]);
				h2_score.appendChild(score);
				div_score.appendChild(h2_score);
				score_list.appendChild(div_score);
			}
		})
}
function create_lobby(){
	var user_name = document.getElementById("create_room_player_name").value;
	var room_name = document.getElementById("create_room_name").value;
	if (room_name && user_name){
		fetch("./beta/create_room/"+room_name, {method: 'POST'})
			.then(response => response.json())
			.then(room_id => {
				console.log(room_id);
				 if (!room_id.hasOwnProperty("error")){
					 fetch("./beta/insert_player/"+room_id+"/"+user_name, {method: 'POST'})
						 .then(response => response.json())
						 .then(player_id =>{
							 console.log(player_id);
							 window.location.href = "./beta/lobby/"+room_id+"/"+player_id
						 })
				 }
			} )
	}
}
function join_lobby(){
	var user_name = document.getElementById("join_room_player_name").value;
	var room_id = document.getElementById("join_room_id").value;
	if (user_name && room_id){
		fetch("./beta/insert_player/"+room_id+"/"+user_name, {method: 'POST'})
			.then(response => response.json())
			.then(player_id => {
				console.log(player_id);
			  window.location.href = "./beta/lobby/"+room_id+"/"+player_id
			})
	}
}
function check_guess()
{
	var textbox = document.getElementById("eval_answer"); 
	var room_id = document.getElementById("room_id").innerHTML;
	var player_id = document.getElementById("player_id").innerHTML;
	console.log(room_id);
	var word = textbox.value;
	if (word){
	fetch("/eval_guess/"+room_id+"/"+ player_id + "/"+word, {method: 'POST'})
		.then(response => response.json())
	  .then(data => {
			var data = JSON.parse(data);
			console.log(data);
			update_results(data);
		});
	textbox.value = '';	
	}
}

function update_results(data){
	var results = document.getElementById("guesses");
	delete_children(results);
	var result_table = document.createElement("table");
	result_table.className = "center";
	var count = 0;
	for (idx in data.valid_guesses) {
		if (count % 5 == 0){
			var row = document.createElement("tr");
		}
		var column = document.createElement("td");
		column.style.padding = "20px";
		var tag = document.createElement("h2");
		var word = data.valid_guesses[idx][0];
		var del = document.createElement("del");
		var text = document.createTextNode(word);
		if (!data.valid_guesses[idx][1]){
			var text = document.createTextNode(" " + word + " ");
			del.appendChild(text);
			tag.appendChild(del);
		}
		else{

			tag.appendChild(text);
		}
		column.appendChild(tag);
		row.appendChild(column);
		count++;
		if (count % 5 == 0){
			result_table.appendChild(row);
		}
	result_table.appendChild(row);
	results.appendChild(result_table);
	document.getElementById("score_box").innerHTML = "Score - " + data.score;
	}
}

function delete_children(node){
	while (node.firstChild){
		node.removeChild(node.lastChild)
	}
}
function start_timer()
{
		var timer_len = new Date().getTime()+(2*60*1000);
	var x = setInterval( () => {
				var now = new Date().getTime();
				var distance = timer_len - now;
				var minutes = Math.floor(distance/(1000*60));
				var seconds = Math.floor((distance % (1000*60))/1000);
				if (seconds < 10){
						document.getElementById("timery").innerHTML = "Time remaining - 0" + minutes + ":0" + seconds;
				}
				else{
						document.getElementById("timery").innerHTML = "Time remaining - 0" + minutes + ":" + seconds;
				}
				if (distance < 0){
						clearInterval(x);
						document.getElementById("timery").innerHTML = "TIME'S UP NERDS";
						reveal("solution_zone");
						reveal("submit_form");
				}
		})
}
function and_then_deactivate(butt_ID)
{
		document.getElementById(butt_ID).disabled = 'disabled';
		document.getElementById(butt_ID).src = "/letters/"+butt_ID+"_pressed.png" ;
}
function focus_on(elem_ID){
	document.getElementById(elem_ID).focus();
}
