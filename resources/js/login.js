import {ThrowError} from "./throwError.js";
import {sendRequest} from "./sendRequest.js";
import {addCookie} from "./addCookie.js";


document.addEventListener('DOMContentLoaded', () => {
    //Методы отправки данных для авторизации
    const _button_signup = document.getElementById("signup-button");
    const _button_login = document.getElementById("login-button");

    _button_signup.addEventListener("click", () => {
        let _signup = {
            login: document.getElementById("sinname").value,
            email: document.getElementById("sinemail").value,
            password: document.getElementById("sinpass").value
        }
        sendRequest("POST", "/api/register", _signup)
            .then(data => RedirectFromLogin("/profile", data['token']))
            .catch(err => ThrowError(err))
    })

    _button_login.addEventListener("click", () => {
        let _login = {
            login: document.getElementById("logname").value,
            password: document.getElementById("logpass").value
        }

        sendRequest("POST", "/api/auth", _login)
            .then(data => RedirectFromLogin("/", data['token']))
            .catch(err => ThrowError(err))
    })

    function RedirectFromLogin(url, cookie_value) {
        addCookie('Authenticate', cookie_value)
        window.location.replace(url)
    }

});