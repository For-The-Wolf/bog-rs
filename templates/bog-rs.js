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
	var word = document.getElementById("eval_answer").value;
	fetch("./eval_guess/"+word)
		.then(response => response.json())
		.then(data => {
			document.getElementById("test").innerHTML = data.words;
		});
}
function start_timer()
{
		var timer_len = new Date().getTime()+(2*60*1000);
		var x = setInterval(function() {
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
				}
		})
}
function and_then_deactivate(butt_ID)
{
		document.getElementById(butt_ID).disabled = 'disabled';
		document.getElementById(butt_ID).src = "letters/"+butt_ID+"_pressed.png" ;
}
