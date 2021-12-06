$(document).ready(function(){
    let counter = $('#counter')
    let time = 5
    setInterval(() => {
        time -= 1
        counter.text("You will be redirected in " + time.toString() + " seconds...")
        if (time < 1) {
            window.location.replace(window.location.origin);
        }
    }, 1000);

    // Remove the timer? Probably doesn't matter
});