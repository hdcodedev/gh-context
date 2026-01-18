# Add connectivity check feature for troubleshooting server connections 

URL: https://github.com/home-assistant/android/pull/6237

## Body

<!--
    Please review the contributing guide before submitting: https://developers.home-assistant.io/docs/android/submit
    Please, complete the following sections to help the processing and review of your changes.
    Please, DO NOT DELETE ANY TEXT from this template! (unless instructed).

    Thank you for submitting a Pull Request and helping to improve Home Assistant. You are amazing!
-->

## Summary
#Fixes #6006
Implement connectivity check in the onboarding error screen.

## Checklist
<!--
    Put an `x` in the boxes that apply. You can also fill these out after
    creating the PR. If you're unsure about any of them, don't hesitate to ask.
    We're here to help! This is simply a reminder of what we are going to look
    for before merging your code.
-->

- [x] New or updated tests have been added to cover the changes following the testing [guidelines](https://developers.home-assistant.io/docs/android/testing/introduction).
- [x] The code follows the project's [code style](https://developers.home-assistant.io/docs/android/codestyle) and [best_practices](https://developers.home-assistant.io/docs/android/best_practices).
- [x] The changes have been thoroughly tested, and edge cases have been considered.
- [x] Changes are backward compatible whenever feasible. Any breaking changes are documented in the changelog for users and/or in the code for developers depending on the relevance.

## Screenshots
<!--
    If this is a user-facing change not in the frontend, please include screenshots in light and dark mode.

    Note: Remove this section if there are no screenshots.
-->

## Development device (reference only)

<img width="250" alt="Development device" src="https://github.com/user-attachments/assets/462d7d6a-dca8-4fcc-9643-bbe69ebf9219" />



---

## UI screenshots

<img width="250" height="2400" alt="image" src="https://github.com/user-attachments/assets/c5b46e24-5458-440d-ad8c-2d90683f3718" />

<img width="250" height="2400" alt="image" src="https://github.com/user-attachments/assets/8b90cdc0-f991-462b-832a-ed9e1cb30eba" />

<img width="250" height="2400" alt="image" src="https://github.com/user-attachments/assets/b97b3acd-be46-47f9-ae85-38a0798e2da0" />

<img width="250" height="2400" alt="image" src="https://github.com/user-attachments/assets/be02b448-9f53-45f4-ad2a-3b5712d07d5f" />

<img width="250" height="2400" alt="image" src="https://github.com/user-attachments/assets/842f7bcb-6780-491d-a7c4-038e0493e572" />

<img width="250" height="2400" alt="image" src="https://github.com/user-attachments/assets/fd3cfcb5-834c-4341-95cb-13061fcf4525" />


## Demo
https://github.com/user-attachments/assets/710f4113-3018-4b33-9576-b3e7d94f5125





## Comments

### Comment 1 by TimoPtr
_2026-01-08T07:50:00Z_

It looks very promising from the video (I didn't look at the code), I would adjust a little bit so that the checks are directly integrated within `More details` instead of adding a button. Also from the settings I'm not expecting to be able to test random URLs, only the one we "know".

For now the main WebView does not have a proper error screen, but I'm expecting it to look like the one in the onboarding. Once everything is agreed on it would be nice to add a section in the troubleshooting section of the doc to go in this page for more information.

---

### Comment 2 by hdcodedev
_2026-01-09T09:23:21Z_

> It looks very promising from the video (I didn't look at the code), I would adjust a little bit so that the checks are directly integrated within `More details` instead of adding a button. Also from the settings I'm not expecting to be able to test random URLs, only the one we "know".
> 
> For now the main WebView does not have a proper error screen, but I'm expecting it to look like the one in the onboarding. Once everything is agreed on it would be nice to add a section in the troubleshooting section of the doc to go in this page for more information.

Thanks @TimoPtr for the input , it really helps a lot. I’ll implement an inline check for the `ConnectionErrorScreen`.
I’ll also clean up the PR and keep it minimal for the first version.
For other cases, such as the troubleshooting section, it might be better to open a separate PR so it’s easier to review. (We can do after we agree with the changes on the `ConnectionErrorScreen` screen )


---

### Comment 3 by TimoPtr
_2026-01-12T13:21:37Z_

@hdcodedev did you sign the CLA or it is the bot not catching it?

---

### Comment 4 by hdcodedev
_2026-01-12T14:01:18Z_

Yes, I have signed the CLA.

---

### Comment 5 by hdcodedev
_2026-01-13T16:06:44Z_

@TimoPtr  , thank you for such a great review. I think I’ve addressed all the issues now, so please take another look when you have time.

I’ve also updated the PR description with new screenshots and a demo.
Additionally, I have a few more suggestions, but I think it would be better to handle those in a separate PR.

---

### Comment 6 by hdcodedev
_2026-01-15T19:41:36Z_

# Update 
https://github.com/user-attachments/assets/835a94ed-2635-484a-8265-deae28a3f616



---

## Timeline Events

- **reviewed** by **unknown** at -
- **labeled** by **home-assistant[bot]** at 2026-01-06T23:09:33Z
- **labeled** by **home-assistant[bot]** at 2026-01-06T23:37:28Z
- **unlabeled** by **home-assistant[bot]** at 2026-01-06T23:37:29Z
- **unlabeled** by **home-assistant[bot]** at 2026-01-06T23:37:30Z
- **labeled** by **home-assistant[bot]** at 2026-01-06T23:37:31Z
- **reviewed** by **unknown** at -
- **reviewed** by **unknown** at -
- **commented** by **TimoPtr** at 2026-01-08T07:50:00Z
- **commented** by **hdcodedev** at 2026-01-09T09:23:21Z
- **mentioned** by **TimoPtr** at 2026-01-09T09:23:22Z
- **subscribed** by **TimoPtr** at 2026-01-09T09:23:23Z
- **committed** by **unknown** at -
- **head_ref_force_pushed** by **hdcodedev** at 2026-01-09T19:54:00Z
- **committed** by **unknown** at -
- **renamed** by **hdcodedev** at 2026-01-09T20:02:26Z
- **ready_for_review** by **hdcodedev** at 2026-01-09T20:02:34Z
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **labeled** by **TimoPtr** at 2026-01-12T13:19:10Z
- **unlabeled** by **home-assistant[bot]** at 2026-01-12T13:19:12Z
- **commented** by **TimoPtr** at 2026-01-12T13:21:37Z
- **mentioned** by **hdcodedev** at 2026-01-12T13:21:38Z
- **subscribed** by **hdcodedev** at 2026-01-12T13:21:38Z
- **commented** by **hdcodedev** at 2026-01-12T14:01:18Z
- **convert_to_draft** by **TimoPtr** at 2026-01-12T14:02:12Z
- **ready_for_review** by **TimoPtr** at 2026-01-12T14:02:21Z
- **reviewed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **convert_to_draft** by **hdcodedev** at 2026-01-13T13:14:02Z
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **ready_for_review** by **hdcodedev** at 2026-01-13T16:03:05Z
- **commented** by **hdcodedev** at 2026-01-13T16:06:44Z
- **mentioned** by **TimoPtr** at 2026-01-13T16:06:45Z
- **subscribed** by **TimoPtr** at 2026-01-13T16:06:45Z
- **review_requested** by **hdcodedev** at 2026-01-13T16:06:51Z
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **reviewed** by **unknown** at -
- **reviewed** by **unknown** at -
- **reviewed** by **unknown** at -
- **reviewed** by **unknown** at -
- **convert_to_draft** by **hdcodedev** at 2026-01-14T12:50:40Z
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **ready_for_review** by **hdcodedev** at 2026-01-15T11:41:59Z
- **review_requested** by **hdcodedev** at 2026-01-15T11:42:06Z
- **reviewed** by **unknown** at -
- **reviewed** by **unknown** at -
- **reviewed** by **unknown** at -
- **reviewed** by **unknown** at -
- **reviewed** by **unknown** at -
- **reviewed** by **unknown** at -
- **convert_to_draft** by **hdcodedev** at 2026-01-15T17:51:11Z
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **ready_for_review** by **hdcodedev** at 2026-01-15T18:56:52Z
- **review_requested** by **hdcodedev** at 2026-01-15T18:57:29Z
- **committed** by **unknown** at -
- **convert_to_draft** by **hdcodedev** at 2026-01-15T19:33:19Z
- **ready_for_review** by **hdcodedev** at 2026-01-15T19:33:28Z
- **commented** by **hdcodedev** at 2026-01-15T19:41:36Z
- **reviewed** by **unknown** at -
- **labeled** by **jpelgrom** at 2026-01-15T21:07:40Z
- **unlabeled** by **home-assistant[bot]** at 2026-01-15T21:07:42Z
- **review_dismissed** by **jpelgrom** at 2026-01-15T21:07:52Z
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **committed** by **unknown** at -
- **reviewed** by **unknown** at -
- **convert_to_draft** by **hdcodedev** at 2026-01-15T21:39:30Z
- **committed** by **unknown** at -
- **ready_for_review** by **hdcodedev** at 2026-01-16T11:56:41Z
- **committed** by **unknown** at -
- **review_requested** by **hdcodedev** at 2026-01-16T12:01:44Z
- **convert_to_draft** by **hdcodedev** at 2026-01-16T12:06:38Z
- **ready_for_review** by **hdcodedev** at 2026-01-16T12:06:46Z
- **reviewed** by **unknown** at -
- **committed** by **unknown** at -
- **review_requested** by **hdcodedev** at 2026-01-16T21:48:16Z
- **convert_to_draft** by **hdcodedev** at 2026-01-17T17:35:05Z
- **ready_for_review** by **hdcodedev** at 2026-01-17T17:35:12Z
- **committed** by **unknown** at -
