# Add support for setting the scroll speed of the mouse wheel.

URL: https://github.com/brave/brave-browser/issues/5717

## Body

Dear developers,
    I actively use Brave browser to work on different operating systems. There is one significant problem
which interfere with productive use of this product. The scroll speed on Linux is very slow and I can't configure it.
System settings or graphical environment settings don't affect GTK applications like Brave.
There are two ways to solve this problem: imwheel and "Chromium Wheel Smooth Scroller" extension.
Unfortunately, these solutions don't completely solve problem, but create additional inconvenience.
    Web-pages are a specific type of document and you often need to look faster through them. 
That's why scroll speed in browser needs to be higher than in other programs.
    Could you please add a setting, which provides ability to change scroll speed by mouse wheel?
Thank you in advance!

## Comments

### Comment 1 by dackdel
_2019-11-27T10:04:45Z_

i second what ^ said. my scroll fingers hurt

---

### Comment 2 by kiklop74
_2020-01-20T19:55:20Z_

Same thing

---

### Comment 3 by buckley310
_2020-02-18T18:08:11Z_

This would be a big win in my opinion. Chromium (and thus chromium-based browsers) used to have a command line flag for this, but it was removed, ostensibly because they think it's GTK's problem.
https://bugs.chromium.org/p/chromium/issues/detail?id=154776
Regardless of where the correct place to solve this is, it remains unsolved for 7 years.

This is keeping me from even giving Brave a fair shot as my primary browser.

---

### Comment 4 by rebron
_2020-02-18T23:00:19Z_

Looks like fix for this is being tracked here: https://bugs.launchpad.net/gtk/+bug/124440

What's insufficient about: https://chrome.google.com/webstore/detail/chromium-wheel-smooth-scr/khpcanbeojalbkpgpmjpdkjnkfcgfkhb?hl=en-US 

Supporting or bringing back --scroll-pixels resolves this for everyone? Because I don't see us doing any UI work for this.

---

### Comment 5 by buckley310
_2020-02-19T01:36:17Z_

> Looks like fix for this is being tracked here: https://bugs.launchpad.net/gtk/+bug/124440

It's being tracked in several places, and if you follow all the finger-pointing the consensus seems to be that it's the windowing compositor's problem, so for X11 the solution is this xf86-libinput driver patch (or some revised version of it):
https://gitlab.freedesktop.org/xorg/driver/xf86-input-libinput/merge_requests/12
however when i applied that patch lots of other software started scrolling uncomfortably quickly, (including brave accessed via remote desktop!)

> What's insufficient about: https://chrome.google.com/webstore/detail/chromium-wheel-smooth-scr/khpcanbeojalbkpgpmjpdkjnkfcgfkhb?hl=en-US

With my settings it negatively affects touchpad use, so requires manual toggling If I'm switching peripherals on my laptop.
It's extra configuration and something to install when i set up a new machine, extra configuration that I don't have to do currently.
I just generally hate the idea of injecting javascript into all webpages that takes over my scroll wheel.

> Supporting or bringing back --scroll-pixels resolves this for everyone? Because I don't see us doing any UI work for this.

My impression is that --scroll-pixels only affects wheel "clicks", not high-resolution-scrolling devices. Does anyone know if this is accurate? If so, that would at least make me happy. My experience has been that high-res touchpad behavior is consistent across OSs and only wheel clicks differ.

---

### Comment 6 by fgimian
_2020-03-30T01:28:34Z_

Just some thoughts about this on my end too.  I'm using Ubuntu 19.10 in a VM with VMware Workstation with a Logitech MX705 mouse.

In Firefox, there are various `about:config` settings which I'm setting as follows:

```javascript
user_pref("mousewheel.system_scroll_override_on_root_content.enabled", true);
user_pref("mousewheel.system_scroll_override_on_root_content.horizontal.factor", 150);
user_pref("mousewheel.system_scroll_override_on_root_content.vertical.factor", 150);
```

These make Firefox absolutely perfect on the scrolling front.

As mentioned Brave (and all Chromium-based browsers) have very slow scrolling on Linux out of the box.  Even when using the extension linked above, the scrolling is not anywhere near as responsive as Firefox.

Often when I move the scroll wheel slowly (one click), Chrome / Brave don't scroll at all while Firefox always does.

The extension sadly is not the answer here, it doesn't work well enough imho

Another point is that many other apps (e.g. Sublime Text, VS Code, Terminal) all scroll really well and much faster relative to Chromium.  So a global GTK setting won't really suffice.

Having an option in Preferences of Brave would be an absolutely lifesaver for all Linux users.  Alternatively the default mouse wheel speed could be adjusted to be more like Windows or macOS out of the box.

Huge love and thanks
Fotis

---

### Comment 7 by buckley310
_2020-03-30T02:24:24Z_

> Another point is that many other apps (e.g. Sublime Text, VS Code, Terminal) all scroll really well and much faster relative to Chromium. So a global GTK setting won't really suffice.

In my experience, VScode actually scrolls the same number of pixels as brave and chromium, however VScode lets you change it.

---

### Comment 8 by fgimian
_2020-03-30T03:18:55Z_

> > Another point is that many other apps (e.g. Sublime Text, VS Code, Terminal) all scroll really well and much faster relative to Chromium. So a global GTK setting won't really suffice.
> 
> In my experience, VScode actually scrolls the same number of pixels as brave and chromium, however VScode lets you change it.

Sorry yep, you're spot on 😄 

---

### Comment 9 by busvw
_2020-04-04T23:32:32Z_

> 
> In Firefox, there are various `about:config` settings which I'm setting as follows:
> 
> ```js
> user_pref("mousewheel.system_scroll_override_on_root_content.enabled", true);
> user_pref("mousewheel.system_scroll_override_on_root_content.horizontal.factor", 150);
> user_pref("mousewheel.system_scroll_override_on_root_content.vertical.factor", 150);
> ```
> 
> These make Firefox absolutely perfect on the scrolling front.
> 
Thank you! 
This issue was really getting on my nerves (the ones in my right hand to be specific). 
Looks like I will be switching back to Firefox. 


---

### Comment 10 by Goddard
_2020-05-11T18:29:49Z_

Big problem to me on a 4k device in Red Hat.  Scrolling too slow.  Need to adjust this setting based on resolution/scale factor, or let the user change it or both.

---

### Comment 11 by daniel-scatigno
_2020-08-11T22:22:43Z_

There is a option on flags:config
Percent-based Scrolling
If enabled, mousewheel and keyboard scrolls will scroll by a percentage of the scroller size. – Mac, Windows, Linux, Chrome OS, Android

But there is no where to set the percentage

---

### Comment 12 by Bader-Al
_2020-11-14T11:17:17Z_

Here's what i do to get around this issue.
1 - Install Imwheel and set up ~/.imwheelrc
2 - In brave go to `brave://flags` 
3 - Disable the following features

-  Smooth Scrolling
-  Percent-based Scrolling

![Screenshot from 2020-11-14 14-16-45](https://user-images.githubusercontent.com/57584896/99145921-14b6e080-2684-11eb-96ae-15eee3c62b1a.png)


---

### Comment 13 by buckley310
_2020-11-14T20:27:30Z_

I have actually just been living with a libinput patch for the past few months. I think it yields the best result out of all the workarounds, although the distro I use makes applying patches on an ongoing basis pretty easy. https://github.com/buckley310/nixos-config/blob/master/modules/scroll-boost/libinput.patch

It is unfortunate that the chromium codebase seems to have changed a lot since "--scroll-pixels" was removed, and it does not seem trivial to revert.

---

### Comment 14 by AlessandroDiGioacchino
_2020-11-15T21:51:35Z_

I'd love to see this happening.
My current workaround is using `libinput-multiplier` [from AUR](https://aur.archlinux.org/packages/libinput-multiplier/), and wrapping the `.desktop` file in `/usr/share/applications` (KDE) with `echo 3 > /tmp/libinput_discrete_deltay_multiplier;` and `;echo 1 > /tmp/libinput_discrete_deltay_multiplier`. Still, not the best solution consindering `libinput-multiplier` affects the entire OS as long as Brave is running (and, in my case, it's running basically always).

---

### Comment 15 by sojusnik
_2020-11-19T21:10:36Z_

As a temporary workaround, hopefully this gets fixed soon, I'm using [this](https://chrome.google.com/webstore/detail/scroll-speed/mfmhdfkinffhnfhaalnabffcfjgcmdhl) simple extension to adjust the scroll speed.

---

### Comment 16 by carlosalop
_2020-12-07T03:04:16Z_

> I'd love to see this happening.
> My current workaround is using `libinput-multiplier` [from AUR](https://aur.archlinux.org/packages/libinput-multiplier/), and wrapping the `.desktop` file in `/usr/share/applications` (KDE) with `echo 3 > /tmp/libinput_discrete_deltay_multiplier;` and `;echo 1 > /tmp/libinput_discrete_deltay_multiplier`. Still, not the best solution consindering `libinput-multiplier` affects the entire OS as long as Brave is running (and, in my case, it's running basically always).

Thank you for this comment, great workaround for now, this was driving me crazy and away from brave and chromium based browsers in general

---

### Comment 17 by tuananhlai
_2021-10-28T13:38:24Z_

Is this feature being worked on? It would be a total game-changer for me if I can increase the scroll speed on Brave.

---

### Comment 18 by buckley310
_2021-11-02T18:58:11Z_

Based on the way things are going, I think this issue will exist until the switch to wayland.

For example, KDE on wayland has a global scroll speed setting that does apply to Brave (I was just testing this earlier today), so for KDE-wayland, this is just not an issue anymore.

---

### Comment 19 by tianer2820
_2022-04-14T23:13:42Z_

I'm having the same issue with Fedora35, running Gnome on Wayland. The mouse scrolls too slow, but the touchpad scroll rate is just fine (and is the same as firefox). I tried running Brave in native Wayland and in Xwayland but that doesn't solve the problem.

---

### Comment 20 by jvillasante
_2022-04-30T15:04:52Z_

Same here, Fedora35 running Gnome on Wayland. Mouse scroll is painfully slow!

---

### Comment 21 by 
_2022-05-01T13:47:19Z_

> Here's what i do to get around this issue. 1 - Install Imwheel and set up ~/.imwheelrc 2 - In brave go to `brave://flags` 3 - Disable the following features
> 
> * Smooth Scrolling
> * Percent-based Scrolling
> 
> ![Screenshot from 2020-11-14 14-16-45](https://user-images.githubusercontent.com/57584896/99145921-14b6e080-2684-11eb-96ae-15eee3c62b1a.png)

Imwheel only work on Xwayland, if you have brave launched with wayland native (ozone settings) it does not work.

> I'd love to see this happening. My current workaround is using `libinput-multiplier` [from AUR](https://aur.archlinux.org/packages/libinput-multiplier/), and wrapping the `.desktop` file in `/usr/share/applications` (KDE) with `echo 3 > /tmp/libinput_discrete_deltay_multiplier;` and `;echo 1 > /tmp/libinput_discrete_deltay_multiplier`. Still, not the best solution consindering `libinput-multiplier` affects the entire OS as long as Brave is running (and, in my case, it's running basically always).

do you know if it possible to use it outside of arch for their git or someting (used arch before so i'm not affraid of terminal) just to know ?

---

### Comment 22 by 
_2022-07-05T11:52:39Z_

> Looks like fix for this is being tracked here: https://bugs.launchpad.net/gtk/+bug/124440
> 
> What's insufficient about: https://chrome.google.com/webstore/detail/chromium-wheel-smooth-scr/khpcanbeojalbkpgpmjpdkjnkfcgfkhb?hl=en-US
> 
> Supporting or bringing back --scroll-pixels resolves this for everyone? Because I don't see us doing any UI work for this.

More and more of linux switch to wayland any news on this ?

---

### Comment 23 by buckley310
_2022-11-26T22:58:27Z_

Looks like the chromium project is raising the default scroll speed in version 109 :partying_face: 
https://bugs.chromium.org/p/chromium/issues/detail?id=1270089#c25

---

### Comment 24 by 
_2022-11-26T23:04:16Z_

> Looks like the chromium project is raising the default scroll speed in version 109 partying_face https://bugs.chromium.org/p/chromium/issues/detail?id=1270089#c25

just hope it will

---

### Comment 25 by lalbers
_2022-11-28T10:25:33Z_

I have tried the latest chromium snapshot (110.0.5546.0) and can confirm that scrolling feels significant better under linux. 🎉
I have been waiting for this for years. https://aur.archlinux.org/packages/chromium-snapshot-bin

---

### Comment 26 by buckley310
_2023-01-12T21:01:09Z_

Yep. Chromium release version 109 more than doubles the scroll speed.

---

### Comment 27 by 
_2023-01-13T17:12:03Z_

Lasted brave (their repo) have fixed the issue for me (fedora).

i so ask to close this issue.

---

### Comment 28 by cheald
_2023-01-22T20:19:43Z_

Brave is definitely scrolling faster now. However, it's uncomfortably fast for me now. I'm using the above linked extension for the time being, but it feels like a giant hack to inject scroll modification into each page. 

I would very much like a flag or setting to be able to set the scroll distance per click myself.

---

### Comment 29 by Septem151
_2023-01-30T18:53:40Z_

Found this issue through a google search, and I second the opinion that the scroll speed after this update has become unbearably fast. Is there any way to revert back to the old scrolling behavior, or to set the scrolling speed in the browser? This feature would be greatly appreciated.

---

### Comment 30 by 
_2023-01-30T19:03:16Z_

> Found this issue through a google search, and I second the opinion that the scroll speed after this update has become unbearably fast. Is there any way to revert back to the old scrolling behavior, or to set the scrolling speed in the browser? This feature would be greatly appreciated.

> Brave is definitely scrolling faster now. However, it's uncomfortably fast for me now. I'm using the above linked extension for the time being, but it feels like a giant hack to inject scroll modification into each page.
> 
> I would very much like a flag or setting to be able to set the scroll distance per click myself.

It's not a brave change but a chromium change, and modifying the settings on brave end would be more complex, it's better to ask this kind of feature directly on chromium (it's why we had to wait a decade to see the slow scroll bug fixed).

---

### Comment 31 by inikishev
_2023-11-17T17:38:59Z_

> Lasted brave (their repo) have fixed the issue for me (fedora).
> 
> i so ask to close this issue.

how do i last brave i have the same issue

---

### Comment 32 by HWXLR8
_2025-06-15T15:53:36Z_

**EDIT**: This was due to brave falling back to X11, needed to explicity launch with the following:
```
brave-browser --ozone-platform=wayland --enable-features=UseOzonePlatform
```

---

Contrary to previous comments, I am still experiencing this issue on ARM64 (raspberry pi 5). When I place chromium and brave side-by-side, a single scroll moves further down the page on chromium compared to brave. 

![Image](https://github.com/user-attachments/assets/7101704e-8e15-473c-bae7-a478b41d270f)

Even more annoyingly, when first gaining focus a single detent of the mouse wheel does nothing on brave, while chromium always responds to the first detent. If I repeatedly lose and regain focus on the window, a single detent scroll wilu perpetually do nothing. 

Browser versions for reference:

Chromium: 
Version 136.0.7103.113 (Official Build) built on Debian GNU/Linux 12 (bookworm) (64-bit)

Brave: 
[Brave 1.79.123 (Official Build) (64-bit)](https://brave.com/latest/)
Chromium: 137.0.7151.104

Seems brave is built from a newer version of chromium.

---

## Timeline Events

- **labeled** by **rebron** at 2019-08-30T16:33:00Z
- **labeled** by **rebron** at 2019-08-30T16:33:04Z
- **commented** by **dackdel** at 2019-11-27T10:04:45Z
- **commented** by **kiklop74** at 2020-01-20T19:55:20Z
- **commented** by **buckley310** at 2020-02-18T18:08:11Z
- **commented** by **rebron** at 2020-02-18T23:00:19Z
- **commented** by **buckley310** at 2020-02-19T01:36:17Z
- **commented** by **fgimian** at 2020-03-30T01:28:34Z
- **commented** by **buckley310** at 2020-03-30T02:24:24Z
- **commented** by **fgimian** at 2020-03-30T03:18:55Z
- **commented** by **busvw** at 2020-04-04T23:32:32Z
- **commented** by **Goddard** at 2020-05-11T18:29:49Z
- **commented** by **daniel-scatigno** at 2020-08-11T22:22:43Z
- **commented** by **Bader-Al** at 2020-11-14T11:17:17Z
- **commented** by **buckley310** at 2020-11-14T20:27:30Z
- **commented** by **AlessandroDiGioacchino** at 2020-11-15T21:51:35Z
- **commented** by **sojusnik** at 2020-11-19T21:10:36Z
- **commented** by **carlosalop** at 2020-12-07T03:04:16Z
- **commented** by **tuananhlai** at 2021-10-28T13:38:24Z
- **commented** by **buckley310** at 2021-11-02T18:58:11Z
- **commented** by **tianer2820** at 2022-04-14T23:13:42Z
- **commented** by **jvillasante** at 2022-04-30T15:04:52Z
- **commented** by **ghost** at 2022-05-01T13:47:19Z
- **commented** by **ghost** at 2022-07-05T11:52:39Z
- **commented** by **buckley310** at 2022-11-26T22:58:27Z
- **commented** by **ghost** at 2022-11-26T23:04:16Z
- **commented** by **lalbers** at 2022-11-28T10:25:33Z
- **commented** by **buckley310** at 2023-01-12T21:01:09Z
- **commented** by **ghost** at 2023-01-13T17:12:03Z
- **commented** by **cheald** at 2023-01-22T20:19:43Z
- **commented** by **Septem151** at 2023-01-30T18:53:40Z
- **commented** by **ghost** at 2023-01-30T19:03:16Z
- **commented** by **inikishev** at 2023-11-17T17:38:59Z
- **added_to_project_v2** by **rebron** at 2024-05-28T17:48:37Z
- **project_v2_item_status_changed** by **rebron** at 2024-05-28T17:48:37Z
- **commented** by **HWXLR8** at 2025-06-15T15:53:36Z
- **unsubscribed** by **buckley310** at 2025-06-16T04:35:47Z
- **subscribed** by **newachu** at 2025-12-05T09:03:48Z
