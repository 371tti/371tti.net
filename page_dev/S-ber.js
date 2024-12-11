
document.addEventListener("readystatechange", () => {

    if (document.readyState === "complete") {
        const loading_spinner = document.querySelector(".loading-spinner");
        loading_spinner.style.animation = "none";
        loading_spinner.style.display = "none";
    }
});
