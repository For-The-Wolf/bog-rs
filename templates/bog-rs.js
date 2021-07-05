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
function update_test()
{
	var textbox = document.getElementById("eval_answer"); 
	var word = textbox.value;
	if (word){
	fetch("./eval_guess/"+word, {method: 'POST'})
		.then(response => response.json())
	  .then(data => {
			console.log(data);
			var results = document.getElementById("guesses");
			delete_children(results);
			for (word in data.words) {
				var tag = document.createElement("h2");
				var text = document.createTextNode(data.words[word]);
				tag.appendChild(text);
				results.appendChild(tag);
			document.getElementById("score_box").innerHTML = "Score - " + data.score;
			}
		});
	textbox.value = '';
		
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
		document.getElementById(butt_ID).src = "letters/"+butt_ID+"_pressed.png" ;
}
