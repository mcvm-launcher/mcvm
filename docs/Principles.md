# MCVM Principles
This is a list of design principles that mcvm as a software and ecosystem will try to follow to the best of its ability.

## 1. Packages are secure
Packages should not be able to perform any privileged actions on a user's system without their explicit consent. Barring bugs and malicious addon files, users should be assured that installed packages will stay in their bounds.

## 2. Packages are self-contained
Packages should be fully functional with only the package file itself and not any other sources, such as data in the repository entry.

## 3. Minecraft content authors should be respected
The people who make content for this game should have their hard work and wishes respected.

## 4. Changes in the environment should be explicit
Your game environment should not change unless you update the profile. Your set of available packages should not change without a repository sync.

## 5. MCVM should be disconnected from any other update schedules
You shouldn't have to update mcvm when a new Minecraft version releases or a new version of a package is published unless the process for using these systems changes.

## 6. MCVM shouldn't touch things it doesn't own
MCVM should not mess with files that it does not manage.

## 7. MCVM should be compatible with every version
The oldest versions should still work just as well as the newest.
