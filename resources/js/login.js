import {ThrowError} from "./throwError.js";
import {sendRequest} from "./sendRequest.js";


document.addEventListener('DOMContentLoaded', () => {
    //Методы отправки данных для авторизации
    let _button_signup = document.getElementById("signup-button");
    let _button_login = document.getElementById("login-button");

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

    function addCookie(name, value) {
        let _expires = new Date(Date.now() + 86400e3 * 0.5);
        document.cookie = encodeURIComponent(name) + '=' + encodeURIComponent(value) + "; expires=" + _expires;
    }

    function deleteCookie(name) {
        document.cookie = encodeURIComponent(name) + "=; max-age=0";
    }

});