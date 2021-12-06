import {sendRequest} from "./sendRequest.js";
import {send_profile} from "./send_profile.js";
import {ThrowError} from "./throwError.js";
import {deleteCookie} from "./deleteCookie.js";

document.addEventListener('DOMContentLoaded', () => {
    const _button_signout = document.querySelector(".signout");

    if (_button_signout != null) {
        _button_signout.addEventListener("click", () => {
            sendRequest("GET", "/logout", null)
                .catch(err => ThrowError(err))
            deleteCookie("Authenticate")
            window.location.replace("/login")
        })
    }

    send_profile(document.querySelector("#profile_name").textContent)

});