export function deleteCookie(name) {
    document.cookie = encodeURIComponent(name) + "=; max-age=0";
}