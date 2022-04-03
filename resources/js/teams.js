import {sendRequest} from "./sendRequest.js";
import {ThrowError} from "./throwError.js";
import {deleteCookie} from "./deleteCookie.js";

function createTeams(teams) {
    let album = document.getElementById("teams-album")
    let modal_album = document.getElementById("body")
    album.innerHTML = ``
    modal_album.innerHTML = ``
    for (let team of teams) {
        let target_modal = 'teaminfo_' + team.name

        album.innerHTML += `
        <div class="col">
            <button type="button" class="card shadow-sm teamcard" data-bs-toggle="modal"
                    data-bs-target="#${target_modal}" data-bs-teamname="${team.name}">
                <svg class="card-img-top" width="100%" height="175"
                     xmlns="http://www.w3.org/2000/svg" role="img" aria-label="${team.name}"
                     preserveAspectRatio="xMidYMid slice" focusable="false">
                    <rect width="100%" height="100%" fill="rgba(80, 80, 90, 1)"/>
                    <image href="../icons/logo_white.png" x="10%" y="10%" width="80%" height="80%"/>
                    <!-- Fallback to logo -->
                    <filter id="blurfilter" width="100%" height="100%" x="-0%" y="-0%">
                        <fegaussianblur in="SourceGraphic" stdDeviation="10"></fegaussianblur>
                        <feComponentTransfer>
                            <feFuncA type="discrete" tableValues="1 1"/>
                        </feComponentTransfer>
                    </filter>
                </svg>
    
                <div class="card-body text-secondary">
                    <h5 class="card-title teamcard_name">${team.name}</h5>
                    <p class="card-text">${team.short_bio}</p>
                </div>
            </button>
        </div>`

        modal_album.innerHTML +=
            `<div class="modal fade" id="${target_modal}" tabIndex="-1" aria-labelledby="teaminfoLabel" aria-hidden="true">
                <div class="modal-dialog modal-dialog-centered modal-dialog-scrollable modal-xl">
                    <div class="modal-content">
                        <button type="button" id="teaminfoClose" class="btn-close" data-bs-dismiss="modal"
                                aria-label="Close"></button>
                        <div class="modal-header">
                            <h5 class="modal-title" id="teaminfoLabel">${team.name}</h5>
                        </div>
                        <div class="modal-body container">
                            <div class="col">
                                <div class="row">
                                    <div class="col">
                                        <svg id="teaminfoImage" width="100%" height="100%"
                                             xmlns="http://www.w3.org/2000/svg" role="img" aria-label="${team.name}"
                                             preserveAspectRatio="xMidYMid slice" focusable="false">
                                            <rect width="100%" height="100%" fill="rgba(80, 80, 90, 1)"/>
                                            <image href="../icons/logo_white.png" x="10%" y="10%" width="80%" height="80%"/>
                                            <!-- Fallback to logo -->
                                            <filter id="blurfilter" width="100%" height="100%" x="-0%" y="-0%">
                                                <fegaussianblur in="SourceGraphic" stdDeviation="10"></fegaussianblur>
                                                <feComponentTransfer>
                                                    <feFuncA type="discrete" tableValues="1 1"/>
                                                </feComponentTransfer>
                                            </filter>
                                        </svg>
                                    </div>
                                    <div class="col">
                                        <div class="p-2 gy-2 row-md-auto desc-blocks" style="word-break: break-word">
                                            ${team.long_bio}
                                        </div>
                                        <div class="p-2 gy-2 row-md-auto desc-blocks" id="teamTags">
                                            Tags: ` + generateComps(team) + `
                                        </div>
                                    </div>
                                </div>
                                <div class="row" style="margin-top: 1rem;">
                                    <div class="col">` + generateMembers(team) + `
                                    </div>
                                    <div class="col">
                                        <!-- Найти применение этому пространству -->
                                    </div>
                                </div>
                            </div>
                        </div>
<!--                        <div class="modal-footer justify-content-center">-->
<!--                            <button type="button" class="btn btn-sm btn-outline-secondary">Join</button>-->
<!--                        </div>-->
                    </div>
                </div>
            </div>`
    }


    function generateComps(team) {
        let output = ''
        for (const comp of team["competences"]) {
            output += comp + ", "
        }
        return output
    }

    function generateMembers(team) {
        let output = ''
        for (const member of team["members"]) {
            output += `<div class="row user"><a href="${'/user/'.concat(member)}"><img src="../icons/logo_white.png"
                                                                     width="30rem"
                                                                     height="30rem">${member}</a></div>`
        }
        return output
    }
}

function generatePagination(page_num, all_pages) {
    let pagination = document.getElementById("pagination-album")
    pagination.innerHTML = `
            <a href="#" id="prev">
                <li><</li>
            </a>`

    let brace = all_pages > 6 ? 6 : all_pages
    for (let i = 1; i <= brace; i++) {
        pagination.innerHTML += `
        <a class = "${i === page_num ? 'is-active' : ''} pg-${i}" href="#">
        <li>${i}</li>
        </a>
        `
    }

    if (all_pages > 6) {
        pagination.innerHTML += ` 
           <a href="#">
                <li>...</li>
            </a>
    `
    }
    pagination.innerHTML += `
            <a href="#" id="next">
                <li>></li>
            </a>
    `
    document.getElementById("prev").addEventListener("click", () => {
        if (page_num > 1) {
            loadPage(page_num - 1)
        }
    })

    document.getElementById("next").addEventListener("click", () => {
        if (page_num < all_pages) {
            loadPage(page_num + 1)
        }
    })

    for (let i = 1; i <= brace; i++) {
        let selector = ".pg-" + i

        document.querySelector(selector).addEventListener("click", () => {
            loadPage(i)
        })
    }


}


function loadPage(page_num) {
    sendRequest("GET", "/api/get_all_teams?page=" + page_num, null)
        .then(response => {
            createTeams(response)
        })
        .catch(err => ThrowError(err))

    sendRequest("GET", "/api/get_teams_pagination", null)
        .then(response => {
            generatePagination(page_num, response)
        })
        .catch(err => ThrowError(err))
}


document.addEventListener('DOMContentLoaded', () => {
    let teams = [{}]
    let page = 1
    let pages_count = 0


    sendRequest("GET", "/api/get_all_teams?page=" + page, null)
        .then(response => {
            teams = response
            createTeams(teams)
        })
        .catch(err => ThrowError(err))

    sendRequest("GET", "/api/get_teams_pagination", null)
        .then(response => {
            pages_count = response
            generatePagination(page, pages_count)
        })
        .catch(err => ThrowError(err))


    const _button_signout = document.querySelector(".signout");

    if (_button_signout != null) {
        _button_signout.addEventListener("click", () => {
            sendRequest("GET", "/logout", null)
                .catch(err => ThrowError(err))
            deleteCookie("Authenticate")
            window.location.replace("/login")
        })
    }
})